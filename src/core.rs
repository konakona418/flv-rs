use crate::exchange::{Destination, ExchangeRegistrable, Packed, PackedContent};
use std::collections::VecDeque;
use std::sync::mpsc;

pub struct Core {
    channel_exchange: Option<mpsc::Sender<Packed>>,
    channel_receiver: mpsc::Receiver<PackedContent>,
    channel_sender: mpsc::Sender<PackedContent>,
}

impl Core {
    pub fn new() -> Self {
        let (channel_sender, channel_receiver) = mpsc::channel();
        Self {
            channel_exchange: None,
            channel_receiver,
            channel_sender,
        }
    }

    pub fn push_data_to_decoder(&self, data: &mut VecDeque<u8>) {

    }

    pub fn start_decoding(&self) {
        // todo: when the video stream is chunked, it's necessary to 'wait' for the next chunk than simply break the decoder loop.
    }
}

impl ExchangeRegistrable for Core {
    fn set_exchange(&mut self, sender: mpsc::Sender<Packed>) {
        self.channel_exchange = Some(sender);
    }

    fn get_sender(&self) -> mpsc::Sender<PackedContent> {
        self.channel_sender.clone()
    }

    fn get_self_as_destination(&self) -> Destination {
        Destination::Core
    }
}

