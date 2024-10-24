use std::sync::mpsc;
use crate::exchange::{ExchangeRegistrable, Packed, PackedContent};

pub struct Remuxer {
    channel_exchange: Option<mpsc::Sender<Packed>>,
    channel_receiver: mpsc::Receiver<PackedContent>,
    channel_sender: mpsc::Sender<PackedContent>,
}

impl ExchangeRegistrable for Remuxer {
    fn set_exchange(&mut self, sender: mpsc::Sender<Packed>) {
        self.channel_exchange = Some(sender);
    }

    fn get_sender(&self) -> mpsc::Sender<PackedContent> {
        self.channel_sender.clone()
    }

    fn get_self_as_destination(&self) -> crate::exchange::Destination {
        crate::exchange::Destination::Remuxer
    }
}