use mursten::{Backend, Data, Updater};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use super::properties::{GetProperties, Property, Value};

pub fn create_repl() -> (Client, Server) {
    let (client_tx, server_rx) = channel();
    let (server_tx, client_rx) = channel();
    (
        Client {
            tx: client_tx,
            rx: client_rx,
        },
        Server {
            tx: server_tx,
            rx: server_rx,
        },
    )
}

pub struct Client {
    tx: Sender<Request>,
    rx: Receiver<Response>,
}

pub struct Server {
    rx: Receiver<Request>,
    tx: Sender<Response>,
}

pub enum Request {
    Set(String, Value),
    Get(String),
    Exit,
}

pub enum Response {
    Ok,
    PropertyNotFound,
    Value(Value),
    ExitSuccessful,
}

impl Client {
    pub fn run(self) {
        let mut rl = Editor::<()>::new();
        loop {
            let res = rl.readline(">> ")
                .map_err(|err| ErrKind::Readline(err))
                .and_then(|line| {
                    let mut words = line.split_whitespace();
                    match words.next() {
                        Some("set") => match words.next() {
                            Some(key) => match parse_value(words.next()) {
                                Some(value) => {
                                    self.tx.send(Request::Set(key.to_string(), value));
                                    self.rx
                                        .recv_timeout(Duration::from_secs(10))
                                        .map_err(|err| match err {
                                            RecvTimeoutError::Timeout => {
                                                ErrKind::Response("No response after 10 seconds.")
                                            }
                                            RecvTimeoutError::Disconnected => ErrKind::Response(
                                                "No response, disconnected from server.",
                                            ),
                                        })
                                        .and_then(|res| match res {
                                            Response::Ok => Ok(()),
                                            Response::PropertyNotFound => {
                                                Err(ErrKind::Response("Property not found."))
                                            }
                                            _ => Err(ErrKind::Response("Unknown response.")),
                                        })
                                }
                                None => Err(ErrKind::Usage("No value provided.")),
                            },
                            None => Err(ErrKind::Usage("No property name provided.")),
                        },
                        Some("get") => match words.next() {
                            Some(key) => {
                                self.tx.send(Request::Get(key.to_string()));
                                self.rx
                                    .recv_timeout(Duration::from_secs(10))
                                    .map_err(|err| match err {
                                        RecvTimeoutError::Timeout => {
                                            ErrKind::Response("No response after 10 seconds.")
                                        }
                                        RecvTimeoutError::Disconnected => ErrKind::Response(
                                            "No response, disconnected from server.",
                                        ),
                                    })
                                    .and_then(|res| match res {
                                        Response::Value(value) => {
                                            println!("{:?}", value);
                                            Ok(())
                                        }
                                        Response::PropertyNotFound => {
                                            Err(ErrKind::Response("Property not found."))
                                        }
                                        _ => Err(ErrKind::Response("Unknown response.")),
                                    })
                            }
                            None => Err(ErrKind::Usage("No property name provided.")),
                        },
                        Some("exit") => {
                            self.tx.send(Request::Exit);
                            self.rx
                                .recv_timeout(Duration::from_secs(10))
                                .map_err(|err| {
                                    match err {
                                        RecvTimeoutError::Timeout => ErrKind::Response("No response after 10 seconds. Use `exit!` to force exit."),
                                        RecvTimeoutError::Disconnected => ErrKind::Response("No response, disconnected from server. Use `exit!` to force exit."),
                                    }
                                })
                                .and_then(|res| match res {
                                    Response::ExitSuccessful => Err(ErrKind::Exit),
                                    _ => Err(ErrKind::Response(
                                        "Wrong response. Use `exit!` to force exit.",
                                    )),
                                })
                        }
                        Some("exit!") => Err(ErrKind::Exit),
                        Some(_) => Err(ErrKind::Usage("Unknown command.")),
                        None => Ok(()),
                    }
                });

            match res {
                Ok(_) => {}
                Err(err) => match err {
                    ErrKind::Readline(err) => {
                        println!("{:?}!", err);
                        break;
                    }
                    ErrKind::Exit => break,
                    ErrKind::Response(msg) | ErrKind::Usage(msg) => println!("{}", msg),
                },
            }
        }
        enum ErrKind {
            Usage(&'static str),
            Response(&'static str),
            Readline(ReadlineError),
            Exit,
        }
    }
}

fn parse_value(s: Option<&str>) -> Option<Value> {
    s.map_or(None, |s| {
        if let Ok(b) = s.parse() {
            Some(Value::Bool(b))
        } else if let Ok(f) = s.parse() {
            Some(Value::Float(f))
        } else if let Ok(i) = s.parse() {
            Some(Value::Integer(i))
        } else {
            None
        }
    })
}

impl<B, D> Updater<B, D> for Server
where
    D: Data + GetProperties,
    B: Backend<D>,
{
    fn update(&mut self, backend: &mut B, data: &mut D) {
        let mut ps = data.properties();
        for request in self.rx.try_iter() {
            let response = match request {
                Request::Set(k, v) => match ps.iter_mut().find(|p| p.name() == k) {
                    Some(p) => {
                        p.set(v);
                        Response::Ok
                    }
                    None => Response::PropertyNotFound,
                },
                Request::Get(k) => match ps.iter().find(|p| p.name() == k) {
                    Some(p) => Response::Value(p.get()),
                    None => Response::PropertyNotFound,
                },
                Request::Exit => {
                    self.tx.send(Response::ExitSuccessful);
                    backend.quit();
                    Response::ExitSuccessful
                }
            };
            self.tx.send(response);
        }
    }
}
