
use super::events::transport::*;


pub trait EventEmitter<E> {
    fn connect_to(&mut self, Address<E>);
}

pub trait EventReceiver<E> {
    fn address(&self) -> Address<E>;
}

pub mod simple {

    use mursten::{Backend, Data, Updater};
    use super::transport::*;
    pub use super::EventReceiver;

    pub struct SimpleEventReceiver<H>
    where
        H: EventHandler,
    {
        handler: H,
        mailbox: Mailbox<H::Event>,
    }

    pub trait EventHandler
    where
        Self: Sized,
    {
        type Model: Data;
        type Backend: Backend<Self::Model>;
        type Event: Clone;
        fn handle_event(
            &mut self,
            &mut Self::Backend,
            &mut Self::Model,
            Self::Event
        );
        fn into_updater(self, name: &'static str) -> SimpleEventReceiver<Self> {
            SimpleEventReceiver {
                handler: self,
                mailbox: Mailbox::new(name),
            }
        }
    }

    impl<H> EventReceiver<H::Event> for SimpleEventReceiver<H>
    where 
        H: EventHandler,
    {
        fn address(&self) -> Address<H::Event> {
            self.mailbox.address()
        }
    }

    impl<H> Updater<H::Backend, H::Model> for SimpleEventReceiver<H>
    where
        H: EventHandler,
    {
        fn update(&mut self, backend: &mut H::Backend, data: &mut H::Model) {
            for event in self.mailbox.read() {
                self.handler.handle_event(backend, data, event);
            }
        }
    }
}

pub mod transport {
    use std::sync::mpsc::{channel, Receiver, Sender};

    pub struct Mailbox<E> {
        name: &'static str,
        receiver: Receiver<E>,
        sender: Sender<E>,
    }

    #[derive(Clone)]
    pub struct Address<E> {
        name: &'static str,
        sender: Sender<E>,
    }

    impl<E> Address<E> {
        fn new(name: &'static str, sender: Sender<E>) -> Self {
            Self { name, sender }
        }
        pub fn send(&self, ev: E) {
            self.sender.send(ev)
                .expect(&format!("Failed to send event to {}", self.name));
        }
    }

    #[derive(Clone)]
    pub struct AddressBook<E> {
        addresses: Vec<Address<E>>,
    }

    impl<E> AddressBook<E>
    where 
        E: Clone
    {
        pub fn new() -> Self {
            Self { addresses: Vec::new() }
        }
        pub fn add(&mut self, a: Address<E>) {
            self.addresses.push(a);
        }
        pub fn send(&self, ev: E) {
            for address in self.addresses.iter() {
                address.send(ev.clone());
            }
        }
    }

    impl<E> Mailbox<E> {
        pub fn new(name: &'static str) -> Self {
            let (sender, receiver) = channel();
            Self { name, sender, receiver }
        }
        pub fn address(&self) -> Address<E> {
            Address::new(self.name, self.sender.clone())
        }
        pub fn read(&mut self) -> Vec<E> {
            self.receiver.try_iter().collect()
        }
    }
}

