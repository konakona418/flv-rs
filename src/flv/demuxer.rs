use std::collections::VecDeque;
use crate::exchange::{Destination, ExchangeRegistrable, Packed, PackedContent, PackedContentToDemuxer, PackedContentToRemuxer};
use std::sync::mpsc;
use std::thread::JoinHandle;
use crate::flv::header::FlvHeader;
use crate::flv::meta::RawMetaData;
use crate::flv::tag::{NormalTagBody, Tag, TagBody, TagType};

pub struct Demuxer {
    channel_exchange: Option<mpsc::Sender<Packed>>,
    channel_receiver: mpsc::Receiver<PackedContent>,
    channel_sender: mpsc::Sender<PackedContent>,
    demuxing: bool,

    cache_video_tags: VecDeque<Tag>,
    cache_audio_tags: VecDeque<Tag>,
    cache_script_tags: VecDeque<Tag>,
    cache_metadata: Option<RawMetaData>,
    cache_flv_header: Option<FlvHeader>,
}

impl Demuxer {
    pub fn new() -> Self {
        let (channel_sender, channel_receiver) = mpsc::channel();
        Self {
            channel_exchange: None,
            channel_receiver,
            channel_sender,
            demuxing: false,
            cache_video_tags: VecDeque::new(),
            cache_audio_tags: VecDeque::new(),
            cache_script_tags: VecDeque::new(),
            cache_metadata: None,
            cache_flv_header: None,
        }
    }

    fn set_demuxing(&mut self, flag: bool) {
        self.demuxing = flag;
    }

    fn process_incoming_tag(&mut self, tag: Tag) {
        if tag.tag_type != TagType::Script {
            match tag.tag_type {
                TagType::Audio => {
                    self.cache_audio_tags.push_back(tag);
                }
                TagType::Video => {
                    self.cache_video_tags.push_back(tag);
                }
                _ => {}
            }
        } else {
            if let TagBody::Normal(ref normal) = tag.tag_body {
                if let NormalTagBody::Script(script) = normal {
                    if script.name.data == "onMetaData" {
                        self.cache_metadata = Some(RawMetaData::new(script.clone()));
                        return;
                    }
                }
            }
            self.cache_script_tags.push_back(tag);
        }
    }

    fn send_to_remuxer(&mut self, pack: Packed) -> Result<(), Box<dyn std::error::Error>> {
        match self.channel_exchange
            .as_ref()
            .unwrap()
            .send(pack) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into())
        }
    }

    fn send_from_cache(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(flv_header) = self.cache_flv_header.take() {
            let pack = Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(PackedContentToRemuxer::PushFlvHeader(flv_header)),
            };
            self.send_to_remuxer(pack)?;
        }

        if let Some(metadata) = self.cache_metadata.take() {
            let pack = Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(PackedContentToRemuxer::PushMetadata(metadata)),
            };
            self.send_to_remuxer(pack)?;
        }

        while let Some(audio) = self.cache_audio_tags.pop_front() {
            let pack = Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(PackedContentToRemuxer::PushTag(audio)),
            };
            self.send_to_remuxer(pack)?;
        }

        while let Some(video) = self.cache_video_tags.pop_front() {
            let pack = Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(PackedContentToRemuxer::PushTag(video)),
            };
            self.send_to_remuxer(pack)?;
        }

        while let Some(script) = self.cache_script_tags.pop_front() {
            let pack = Packed {
                packed_routing: Destination::Remuxer,
                packed_content: PackedContent::ToRemuxer(PackedContentToRemuxer::PushTag(script)),
            };
            self.send_to_remuxer(pack)?;
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Ok(received) = self.channel_receiver.recv() {
                match received {
                    PackedContent::ToDemuxer(content) => {
                        match content {
                            PackedContentToDemuxer::PushTag(tag) => {
                                // todo: implement tag processing.
                                self.process_incoming_tag(tag);
                            }
                            PackedContentToDemuxer::PushFlvHeader(flv_header) => {
                                println!("Received flv header.");
                                self.cache_flv_header = Some(flv_header);
                            }
                            PackedContentToDemuxer::StartDemuxing => {
                                println!("Start demuxing.");
                                self.set_demuxing(true);
                            }
                            PackedContentToDemuxer::StopDemuxing => {
                                println!("Stop demuxing.");
                                self.set_demuxing(false);
                            }
                            PackedContentToDemuxer::CloseWorkerThread => {
                                println!("Close worker thread.");
                                break;
                            }
                            PackedContentToDemuxer::Now => {
                                // just to temporarily remove thread blockage.
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                // todo: use a better way instead of recv().
                println!("Channel closed.");
                return Ok(());
            }

            if !self.demuxing {
                continue;
            }

            self.send_from_cache()?;
        }
        Ok(())
    }

    /// Launch a worker thread, move the self into it.
    /// Note that the data stream will not be sent unless the StartDemuxing command is sent.
    pub fn launch_worker_thread(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            self.run().unwrap();
        })
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