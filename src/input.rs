use mursten;
use nalgebra::{Point2, Vector2};

pub trait OnKeyboard {
    fn handle(&mut self, _: KeyboardEvent) {}
}

pub enum KeyboardEvent {
    Pressed(Key, KeyModifiers),
    Released(Key, KeyModifiers),
}

pub struct KeyModifiers {}

pub enum Key {
    A,
    S,
    D,
    F,
    Q,
    W,
    E,
    J,
    K,
}

pub trait OnMouse {
    fn handle(&mut self, _: MouseEvent) {}
}

pub enum MouseEvent {
    Pressed(MouseButton, Point2<f32>),
    Released(MouseButton, Point2<f32>),
    Movement(Vector2<f32>),
    Wheel(Vector2<f32>),
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub struct KeyboardUpdater {}

impl KeyboardUpdater {
    pub fn new() -> Self {
        KeyboardUpdater {}
    }
}

impl<B, D> mursten::Updater<B, D> for KeyboardUpdater
where
    D: mursten::Data + OnKeyboard,
    B: mursten::Backend<D> + backend::KeyboardEventSource,
{
    fn update(&mut self, backend: &mut B, data: &mut D) {
        for event in backend.drain_events() {
            data.handle(event);
        }
    }
}

pub struct MouseUpdater {}

impl MouseUpdater {
    pub fn new() -> Self {
        MouseUpdater {}
    }
}

impl<B, D> mursten::Updater<B, D> for MouseUpdater
where
    D: mursten::Data + OnMouse,
    B: mursten::Backend<D> + backend::MouseEventSource,
{
    fn update(&mut self, backend: &mut B, data: &mut D) {
        for event in backend.drain_events() {
            data.handle(event);
        }
    }
}

// TODO: Maybe implement a GenericUpdater, like an `EventPump<Source, Event, Handler>`.
//
// Generally the problem is alvays the same and the same components appear. There is a _Source_
// from which we can drain events. Then there is _Handler_ which wants to know about those events.
// To operate them there is an _Updater_ (TODO: think about a better name for it... there can be an
// interesting metaphor with a pump) which takes the events from the sources and gives them to the
// Handler. And there is the _Event_ itself.


pub mod backend {
    pub trait KeyboardEventSource {
        fn drain_events(&mut self) -> Vec<super::KeyboardEvent>;
    }

    pub trait MouseEventSource {
        fn drain_events(&mut self) -> Vec<super::MouseEvent>;
    }
}

