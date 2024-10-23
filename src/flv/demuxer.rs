use crate::exchange::{Destination, ExchangeRegistrable, Packed, PackedContent};
use std::fs::Metadata;
use std::sync::mpsc;

pub struct Demuxer {
    pub metadata: Option<Metadata>,
    channel_exchange: Option<mpsc::Sender<Packed>>,
    channel_receiver: mpsc::Receiver<PackedContent>,
    channel_sender: mpsc::Sender<PackedContent>,
}

impl Demuxer {
    pub fn new() -> Self {
        let (channel_sender, channel_receiver) = mpsc::channel();
        Self {
            metadata: None,
            channel_exchange: None,
            channel_receiver,
            channel_sender,
        }
    }
}

impl ExchangeRegistrable for Demuxer {
    fn set_exchange(&mut self, sender: mpsc::Sender<Packed>) {
        self.channel_exchange = Some(sender);
    }

    fn get_sender(&self) -> mpsc::Sender<PackedContent> {
        self.channel_sender.clone()
    }

    fn get_self_as_destination(&self) -> Destination {
        Destination::Demuxer
    }
}