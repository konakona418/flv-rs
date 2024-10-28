use crate::flv::header::FlvHeader;
use crate::flv::meta::RawMetaData;
use crate::flv::tag::Tag;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::mpsc;
use std::thread::JoinHandle;
use crate::fmpeg::remux_context::AudioCodecType;

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

    pub fn register(&mut self, registry: &mut dyn ExchangeRegistrable) {
        registry.set_exchange(self.sender.clone());
        self.channels.insert(registry.get_self_as_destination(), registry.get_sender());
    }

    pub fn process_incoming(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(received) = self.receiver.recv() {
            let routing = received.packed_routing;
            self.channels
                .get(&routing)
                .unwrap()
                .send(received.packed_content)?;
        } else {
            return Err("[Exchange] Channel closed.".into());
        }
        Ok(())
    }

    pub fn launch_worker_thread(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            loop {
                self.process_incoming().unwrap();
            }
        })
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
    Data(RemuxedData),
    DecoderConfig(MseDecoderConfig),
    Command
}

pub enum RemuxedData {
    Header(Vec<u8>),
    Audio(Vec<u8>),
    Video(Vec<u8>)
}

pub enum MseDecoderConfig {
    AudioCodec(AudioCodecConfig),
    VideoCodec(VideoCodecConfig)
}

/// Note: mp3 in video is not supported by some browsers.
/// So in certain circumstances, take special care.
pub struct AudioCodecConfig {
    conf_string: String,

    pub audio_codec_type: AudioCodecType,
    pub audio_object_type: u8,
}

impl AudioCodecConfig {
    pub fn new(codec_type: AudioCodecType, object_type: u8) -> AudioCodecConfig {
        Self {
            conf_string: "".to_string(),
            audio_codec_type: codec_type,
            audio_object_type: object_type
        }
    }

    pub fn audio_conf(&mut self) -> String {
        match self.audio_codec_type {
            AudioCodecType::Aac => {
                if self.conf_string.is_empty() {
                    self.conf_string = format!("mp4a.40.{}", self.audio_object_type);
                }
                self.conf_string.clone()
            }
            AudioCodecType::Mp3 => {
                "mp3".to_string()
            }
            AudioCodecType::None => {
                panic!("No audio codec type specified.")
            }
        }
    }
}

pub struct VideoCodecConfig {
    pub conf_string: String,

    pub avc_profile_indication: u8,
    pub avc_profile_compatibility: u8,
    pub avc_level_indication: u8,
}

impl VideoCodecConfig {
    pub fn new(profile_indication: u8, profile_compatibility: u8, level_indication: u8) -> VideoCodecConfig {
        Self {
            conf_string: "".to_string(),
            avc_profile_indication: profile_indication,
            avc_profile_compatibility: profile_compatibility,
            avc_level_indication: level_indication
        }
    }

    pub fn video_conf(&mut self) -> String {
        self.conf_string = format!("avc1.{:x}{:x}{:x}", self.avc_profile_indication, self.avc_profile_compatibility, self.avc_level_indication);
        self.conf_string.clone()
    }
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