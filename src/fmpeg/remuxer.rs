use crate::exchange::{ExchangeRegistrable, Packed, PackedContent, PackedContentToRemuxer};
use crate::flv::meta::RawMetaData;
use crate::flv::tag::Tag;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::thread::JoinHandle;
use crate::flv::header::FlvHeader;
use crate::fmpeg::remux_context::RemuxContext;

pub struct Remuxer {
    channel_exchange: Option<mpsc::Sender<Packed>>,
    channel_receiver: mpsc::Receiver<PackedContent>,
    channel_sender: mpsc::Sender<PackedContent>,
    remuxing: bool,

    tags: VecDeque<Tag>,
    metadata: Option<RawMetaData>,
    flv_header: Option<FlvHeader>,

    ctx: RemuxContext,
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

impl Remuxer {
    pub fn new() -> Self {
        let (channel_sender, channel_receiver) = mpsc::channel();
        Self {
            channel_exchange: None,
            channel_receiver,
            channel_sender,
            remuxing: false,
            tags: VecDeque::new(),
            metadata: None,
            flv_header: None,
            ctx: RemuxContext::new(),
        }
    }

    #[inline]
    fn set_remuxing(&mut self, flag: bool) {
        self.remuxing = flag;
    }

    pub fn remux(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(tag) = self.tags.pop_front() {
            // todo: remux
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Ok(received) = self.channel_receiver.recv() {
                if let PackedContent::ToRemuxer(content) = received {
                    match content {
                        PackedContentToRemuxer::PushTag(tag) => {
                            self.tags.push_back(tag);
                        }
                        PackedContentToRemuxer::PushFlvHeader(flv_header) => {
                            self.ctx.parse_flv_header(&flv_header);
                            self.flv_header = Some(flv_header);
                        }
                        PackedContentToRemuxer::PushMetadata(metadata) => {
                            self.ctx.parse_metadata(&metadata);
                            self.metadata = Some(metadata);
                        }
                        PackedContentToRemuxer::StartRemuxing => {
                            self.set_remuxing(true)
                        }
                        PackedContentToRemuxer::StopRemuxing => {
                            self.set_remuxing(false)
                        }
                        PackedContentToRemuxer::CloseWorkerThread => {
                            break;
                        }
                        PackedContentToRemuxer::Now => { }
                    }
                }
            } else {
                println!("Channel closed.");
                break;
            }

            if !self.remuxing {
                continue;
            }

            self.remux()?;
        }
        Ok(())
    }

    pub fn launch_worker_thread(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            self.run().unwrap();
        })
    }
}