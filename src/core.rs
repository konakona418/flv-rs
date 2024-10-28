use crate::exchange::{AudioCodecConfig, Destination, ExchangeRegistrable, MseDecoderConfig, Packed, PackedContent, PackedContentToCore, PackedContentToDecoder, PackedContentToDemuxer, PackedContentToRemuxer, RemuxedData, VideoCodecConfig};
use std::collections::VecDeque;
use std::sync::mpsc;

pub struct Core {
    channel_exchange: Option<mpsc::Sender<Packed>>,
    channel_receiver: mpsc::Receiver<PackedContent>,
    channel_sender: mpsc::Sender<PackedContent>,

    pub buffer: VecDeque<RemuxedData>,

    audio_codec_conf: Option<AudioCodecConfig>,
    video_codec_conf: Option<VideoCodecConfig>,
}

impl Core {
    pub fn new() -> Self {
        let (channel_sender, channel_receiver) = mpsc::channel();
        Self {
            channel_exchange: None,
            channel_receiver,
            channel_sender,
            buffer: VecDeque::new(),
            audio_codec_conf: None,
            video_codec_conf: None,
        }
    }

    pub fn process_incoming(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Ok(data) = self.channel_receiver.try_recv() {
            match data {
                PackedContent::ToCore(PackedContentToCore::Data(data)) => {
                    self.buffer.push_back(data);
                },
                PackedContent::ToCore(PackedContentToCore::DecoderConfig(conf)) => {
                    match conf {
                        MseDecoderConfig::AudioCodec(audio_codec) => {
                            self.audio_codec_conf = Some(audio_codec);
                        }
                        MseDecoderConfig::VideoCodec(video_codec) => {
                            self.video_codec_conf = Some(video_codec);
                        }
                    }
                },
                _ => {}
            };
        };
        Ok(())
    }

    pub fn send(&self, packed: Packed) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(_) = self.channel_exchange
            .as_ref()
            .unwrap()
            .send(packed) {
            Err("[Core] Channel closed.".into())
        } else {
            Ok(())
        }
    }

    pub fn push_data_to_decoder(&self, data: &mut VecDeque<u8>) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Decoder,
                packed_content: PackedContent::ToDecoder(
                    PackedContentToDecoder::PushData(data.clone())
                ),
            }
        )
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.start_decoding()?;
        self.start_demuxing()?;
        self.start_remuxing()?;
        Ok(())
    }

    fn start_decoding(&self) -> Result<(), Box<dyn std::error::Error>> {
        // todo: when the video stream is chunked, it's necessary to 'wait' for the next chunk than simply break the decoder loop.
        self.send(
            Packed {
                packed_routing: Destination::Decoder,
                packed_content: PackedContent::ToDecoder(
                    PackedContentToDecoder::StartDecoding
                ),
            }
        )
    }

    fn start_demuxing(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Demuxer,
                packed_content: PackedContent::ToDemuxer(
                    PackedContentToDemuxer::StartDemuxing
                ),
            }
        )
    }

    fn start_remuxing(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(
                    PackedContentToRemuxer::StartRemuxing
                ),
            }
        )
    }

    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.stop_decoding()?;
        self.stop_demuxing()?;
        self.stop_remuxing()?;
        Ok(())
    }

    fn stop_decoding(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Decoder,
                packed_content: PackedContent::ToDecoder(
                    PackedContentToDecoder::StopDecoding
                ),
            }
        )
    }

    fn stop_demuxing(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Demuxer,
                packed_content: PackedContent::ToDemuxer(
                    PackedContentToDemuxer::StopDemuxing
                ),
            }
        )
    }

    fn stop_remuxing(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(
                    PackedContentToRemuxer::StopRemuxing
                ),
            }
        )
    }

    pub fn now(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.decode_now()?;
        self.demux_now()?;
        self.remux_now()?;
        Ok(())
    }

    fn decode_now(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Decoder,
                packed_content: PackedContent::ToDecoder(
                    PackedContentToDecoder::Now
                ),
            }
        )
    }

    fn demux_now(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Demuxer,
                packed_content: PackedContent::ToDemuxer(
                    PackedContentToDemuxer::Now
                ),
            }
        )
    }

    fn remux_now(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(
                    PackedContentToRemuxer::Now
                ),
            }
        )
    }

    pub fn drop_all_workers(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.drop_decoding_worker()?;
        self.drop_demuxing_worker()?;
        self.drop_remuxing_worker()?;
        Ok(())
    }

    fn drop_decoding_worker(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Decoder,
                packed_content: PackedContent::ToDecoder(
                    PackedContentToDecoder::CloseWorkerThread
                ),
            }
        )
    }

    fn drop_demuxing_worker(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Demuxer,
                packed_content: PackedContent::ToDemuxer(
                    PackedContentToDemuxer::CloseWorkerThread
                ),
            }
        )
    }

    fn drop_remuxing_worker(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(
                    PackedContentToRemuxer::CloseWorkerThread
                ),
            }
        )
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

impl IConsumable for Core {
    type ConsumerData = RemuxedData;

    fn consume(&mut self) -> Result<RemuxedData, Box<dyn std::error::Error>> {
        self.process_incoming()?;

        if let Some(data) = self.buffer.pop_front() {
            Ok(data)
        } else {
            Err("No data available".into())
        }
    }
}

impl Core {
    pub fn get_audio_codec_conf(&mut self) -> Option<String> {
        match self.audio_codec_conf {
            Some(ref mut conf) => Some(conf.audio_conf()),
            None => None
        }
    }

    pub fn get_video_codec_conf(&mut self) -> Option<String> {
        match self.video_codec_conf {
            Some(ref mut conf) => Some(conf.video_conf()),
            None => None
        }
    }

    pub fn is_codec_configured(&self) -> bool {
        self.audio_codec_conf.is_some() && self.video_codec_conf.is_some()
    }

    /// Returns the codec configuration if it is already set
    /// Returns a tuple of audio and video codec configuration in String.
    /// If the codec configuration is not set, returns None.
    /// This method will not block.
    pub fn try_get_codec_conf(&mut self) -> Option<(String, String)> {
        if self.is_codec_configured() {
            return Some((self.get_audio_codec_conf()?, self.get_video_codec_conf()?));
        }
        None
    }

    /// Returns the codec configuration. This method will block until the codec configuration is ready.
    pub fn get_codec_conf(&mut self) -> Result<(String, String), Box<dyn std::error::Error>> {
        self.process_incoming()?;

        loop {
            match self.try_get_codec_conf() {
                Some(conf) => return Ok(conf),
                None => self.now()?
            }
        }
    }
}

pub trait IConsumable {
    type ConsumerData;
    fn consume(&mut self) -> Result<Self::ConsumerData, Box<dyn std::error::Error>>;
}

