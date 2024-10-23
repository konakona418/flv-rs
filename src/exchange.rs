use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::mpsc;

pub struct Exchange {
    receiver: mpsc::Receiver<Packed>,
    pub sender: mpsc::Sender<Packed>,

    pub channels: HashMap<Destination, mpsc::Sender<PackedContent>>
}

pub enum Destination {
    Core,
    Decoder,
    Demuxer
}

impl Hash for Destination {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Destination::Core => 0.hash(state),
            Destination::Decoder => 1.hash(state),
            Destination::Demuxer => 2.hash(state)
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
            }
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
            match received.packed_routing {
                PackedRouting::ToCore => {
                    self.channels.get(&Destination::Core).unwrap().send(received.packed_content)?;
                }
                PackedRouting::ToDecoder => {
                    self.channels.get(&Destination::Decoder).unwrap().send(received.packed_content)?;
                }
                PackedRouting::ToDemuxer => {
                    self.channels.get(&Destination::Demuxer).unwrap().send(received.packed_content)?;
                }
            }
        }
        Ok(())
    }
}

pub struct Packed {
    pub packed_routing: PackedRouting,
    pub packed_content: PackedContent
}

pub enum PackedRouting {
    ToCore,
    ToDecoder,
    ToDemuxer,
}

pub enum PackedContent {
    ToCore(PackedContentToCore),
    ToDecoder(PackedContentToDecoder),
    ToDemuxer(PackedContentToDemuxer),
}

pub enum PackedContentToCore {
    Data,
    Command
}

pub enum PackedContentToDecoder {
    PushData(VecDeque<u8>),
    StartDecoding,
    StopDecoding,
    CloseWorkerThread
}

pub enum PackedContentToDemuxer {
    Data,
    Command
}