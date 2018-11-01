use midir::{Ignore, MidiInput, MidiInputConnection};
use mursten::{Data, Updater};
use std::io::{stdin, stdout, Write};
use std::sync::mpsc::{channel, Receiver};

struct MidiHandle {
    receiver: Receiver<MidiMessage>,
    midi_connection: MidiInputConnection<()>,
}

impl MidiHandle {
    fn get_messages(&self) -> Vec<MidiMessage> {
        self.receiver.try_iter().collect()
    }
}

pub trait OnMidiMessage {
    fn on_midi_message(&mut self, message: MidiMessage);
}

#[derive(Debug)]
pub enum MidiMessage {
    NoteOff(u8, u8),
    NoteOn(u8, u8),
    KeyPressure(u8, u8),
    ControlChange(u8, u8),
    ProgramChange(u8),
    ChannelPressure(u8),
    PitchBendChange(u16),
    Start,
    Stop,
}

impl MidiMessage {
    fn from(bytes: &[u8]) -> Option<MidiMessage> {
        let mut bytes = bytes.iter().cloned();
        let b1 = bytes.next()?;
        let msg = match (b1 & 0b1111_0000) >> 4 {
            0b1000 => MidiMessage::NoteOff(bytes.next()?, bytes.next()?),
            0b1001 => {
                let key = bytes.next()?;
                let vel = bytes.next()?;
                if vel > 0 {
                    MidiMessage::NoteOn(key, vel)
                } else {
                    MidiMessage::NoteOff(key, vel)
                }
            }
            0b1010 => MidiMessage::KeyPressure(bytes.next()?, bytes.next()?),
            0b1011 => MidiMessage::ControlChange(bytes.next()?, bytes.next()?),
            0b1100 => MidiMessage::ProgramChange(bytes.next()?),
            0b1101 => MidiMessage::ChannelPressure(bytes.next()?),
            0b1110 => {
                let l = bytes.next()? as u16;
                let h = bytes.next()? as u16;
                let value = h * 128 + l;
                MidiMessage::PitchBendChange(value)
            }
            0b1111 => match b1 {
                0xFA => MidiMessage::Start,
                0xFC => MidiMessage::Stop,
                _ => {
                    return None;
                }
            },
            _ => {
                return None;
            }
        };
        Some(msg)
    }
}

pub struct MidiUpdater {
    midi_handle: MidiHandle,
}

impl MidiUpdater {
    pub fn prompt() -> Self {
        let midi_in = create_midi_input();
        let name = prompt_port(&midi_in);
        Self {
            midi_handle: listen_from_port(midi_in, &name).unwrap(),
        }
    }
    pub fn new(name: &str) -> Self {
        let midi_in = create_midi_input();
        Self {
            midi_handle: listen_from_port(midi_in, name).unwrap(),
        }
    }
}

impl<B, D> Updater<B, D> for MidiUpdater
where
    D: Data + OnMidiMessage,
{
    fn update(&mut self, _: &mut B, data: &mut D) {
        for msg in self.midi_handle.get_messages() {
            data.on_midi_message(msg);
        }
    }
}

fn create_midi_input() -> MidiInput {
    let mut midi_in = MidiInput::new("midi_one input port").unwrap();
    midi_in.ignore(Ignore::None);
    midi_in
}

fn prompt_port(midi_in: &MidiInput) -> String {
    println!("\n# Please connect to a MIDI input\n#");
    println!("# Available input ports:");
    for i in 0..midi_in.port_count() {
        println!("#   {}: {}", i, midi_in.port_name(i).unwrap());
    }
    print!("# Please select input port: ");
    stdout().flush().unwrap();
    let in_port: usize = {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        input.trim().parse().unwrap()
    };
    midi_in.port_name(in_port).unwrap()
}

fn listen_from_port(midi_in: MidiInput, name: &str) -> Result<MidiHandle, ()> {
    let (transmitter, receiver) = channel();

    let port_index = |midi_in: &MidiInput, name| {
        let mut res = None;
        for i in 0..midi_in.port_count() {
            if name == midi_in.port_name(i).unwrap() {
                res = Some(i);
            }
        }
        res
    };

    println!("# \nOpening connection");
    match port_index(&midi_in, name) {
        Some(port_index) => {
            let midi_connection = midi_in
                .connect(
                    port_index,
                    "midir-forward",
                    move |stamp, message, _| {
                        if let Some(message) = MidiMessage::from(message) {
                            transmitter.send(message);
                        }
                    },
                    (),
                )
                .unwrap();
            println!("# Connection open, listening to '{}'", name);
            Ok(MidiHandle {
                receiver,
                midi_connection,
            })
        }
        None => {
            println!("# No port found by the name of '{}'", name);
            Err(())
        }
    }
}
