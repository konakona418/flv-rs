use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::mpsc;
use crate::flv::header::FlvHeader;
use crate::flv::meta::RawMetaData;
use crate::flv::tag::Tag;

pub struct Exchange {
    receiver: mpsc::Receiver<Packed>,
    pub sender: mpsc::Sender<Packed>,

    pub channels: HashMap<Destination, mpsc::Sender<PackedContent>>
}

pub enum Destination {
    Core,
    Decoder,
    Demuxer,
    Remuxer,
}

impl Hash for Destination {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Destination::Core => 0.hash(state),
            Destination::Decoder => 1.hash(state),
            Destination::Demuxer => 2.hash(state),
            Destination::Remuxer => 3.hash(state)
        }
    }
}

impl PartialEq<Self> for Destination {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Destination::Core => match other {
                Destination::Core => true,
                _ => false
            },
            Destination::Decoder => match other {
                Destination::Decoder => true,
                _ => false
            },
            Destination::Demuxer => match other {
                Destination::Demuxer => true,
                _ => false
            },
            Destination::Remuxer => match other {
                Destination::Remuxer => true,
                _ => false
            },
        }
    }
}

impl Eq for Destination { }

pub trait ExchangeRegistrable {
    fn set_exchange(&mut self, sender: mpsc::Sender<Packed>);

    fn get_sender(&self) -> mpsc::Sender<PackedContent>;
    fn get_self_as_destination(&self) -> Destination;
}

impl Exchange {
    pub fn new() -> Exchange {
        let (sender, receiver) = mpsc::channel::<Packed>();
        Exchange {
            receiver,
            sender,
            channels: HashMap::new()
        }
    }

    pub fn get_exchange_sender(&self) -> mpsc::Sender<Packed> {
        self.sender.clone()
    }

    pub fn get_sender(&self, channel_dest: Destination) -> Option<mpsc::Sender<PackedContent>> {
        self.channels.get(&channel_dest).cloned()
    }

    pub fn register(&mut self, mut registry: Box<dyn ExchangeRegistrable>) {
        registry.set_exchange(self.sender.clone());
        self.channels.insert(registry.get_self_as_destination(), registry.get_sender());
    }

    pub fn process_incoming(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(received) = self.receiver.try_recv() {
            let routing = received.packed_routing;
            self.channels
                .get(&routing)
                .unwrap()
                .send(received.packed_content)?;
        }
        Ok(())
    }
}

pub struct Packed {
    pub packed_routing: Destination,
    pub packed_content: PackedContent
}

pub enum PackedContent {
    ToCore(PackedContentToCore),
    ToDecoder(PackedContentToDecoder),
    ToDemuxer(PackedContentToDemuxer),
    ToRemuxer(PackedContentToRemuxer)
}

pub enum PackedContentToCore {
    Data(Vec<u8>),
    Command
}

pub enum PackedContentToDecoder {
    PushData(VecDeque<u8>),

    StartDecoding,
    StopDecoding,
    CloseWorkerThread,

    Now
}

pub enum PackedContentToDemuxer {
    PushTag(Tag),
    PushFlvHeader(FlvHeader),

    StartDemuxing,
    StopDemuxing,
    CloseWorkerThread,

    Now
}

pub enum PackedContentToRemuxer {
    PushTag(Tag),
    PushFlvHeader(FlvHeader),
    PushMetadata(RawMetaData),

    StartRemuxing,
    StopRemuxing,
    CloseWorkerThread,

    Now
}