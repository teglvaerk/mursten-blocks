use cursive::Cursive;
use mursten::{Backend, Data, Renderer};

use super::events::{EventEmitter, EventReceiver};
use super::events::transport::{Mailbox, Address, AddressBook};


/// Reexported dependecies
/// ----------------------

pub use cursive;


/// Structure and trait definition
/// ------------------------------

pub struct CursiveRenderer<V>
where
    V: CursiveView,
{
    view: V,
    context: CursiveContext<V>,
}

pub struct CursiveContext<V>
where
    V: CursiveView,
{
    screen: Cursive,
    address_book: AddressBook<V::Event>, 
    mailbox: Mailbox<V::Event>,
}

pub trait CursiveView
where
    Self: Sized
{
    type Model: Data;
    type Event: Clone;
    fn configure(
        &mut self,
        &mut CursiveContext<Self>,
    );
    fn update(
        &mut self,
        &mut CursiveContext<Self>,
        &Self::Model
    );
    fn need_to_update(&mut self, _: &Self::Model) -> bool {
        true
    }
}


/// Renderer implemtation
/// ---------------------

impl<V> CursiveRenderer<V>
where
    V: CursiveView,
{
    pub fn new(name: &'static str, mut view: V) -> Self {
        let mut context = CursiveContext {
            screen: Cursive::default(),
            address_book: AddressBook::new(),
            mailbox: Mailbox::new(name),
        };
        view.configure(&mut context);
        CursiveRenderer {
            view,
            context,
        }
    }
}

impl<V> CursiveContext<V>
where
    V: CursiveView,
{
    pub fn screen<'a>(&'a mut self) -> &'a mut Cursive {
        &mut self.screen
    }
    pub fn step(&mut self) {
        self.screen.step();
    }
}

impl<B, V> Renderer<B, V::Model> for CursiveRenderer<V>
where
    V: CursiveView,
    B: Backend<V::Model>,
{
    fn render(&mut self, _: &mut B, data: &V::Model) {
        if self.view.need_to_update(data) {
            self.view.update(&mut self.context, data);
        }
        self.context.step();
        self.context.pump_events();
    }
}


/// Event support implemtation
/// --------------------------

impl<V> EventEmitter<V::Event> for CursiveRenderer<V>
where
    V: CursiveView,
{
    fn connect_to(&mut self, addr: Address<V::Event>) {
        self.context.connect_to(addr)
    }
} 

impl<V> EventEmitter<V::Event> for CursiveContext<V>
where
    V: CursiveView,
{
    fn connect_to(&mut self, addr: Address<V::Event>) {
        self.address_book.add(addr);
    }
} 

impl<V> EventReceiver<V::Event> for CursiveContext<V>
where
    V: CursiveView,
{
    fn address(&self) -> Address<V::Event> {
        self.mailbox.address()
    }
}

impl<V> CursiveContext<V>
where 
    V: CursiveView,
{
    fn pump_events(&mut self) {
        for ev in self.mailbox.read() {
            self.address_book.send(ev.clone());
        }
    }
}

