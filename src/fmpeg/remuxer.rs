use crate::exchange::PackedContentToCore::Data;
use crate::exchange::{Destination, ExchangeRegistrable, MseDecoderConfig, Packed, PackedContent, PackedContentToCore, PackedContentToRemuxer, RemuxedData};
use crate::flv::header::FlvHeader;
use crate::flv::meta::RawMetaData;
use crate::flv::tag::{Tag, TagType};
use crate::fmpeg::encoder::{Encoder, DEFAULT_AUDIO_TRACK_ID, DEFAULT_VIDEO_TRACK_ID};
use crate::fmpeg::mp4head::ISerializable;
use crate::fmpeg::parser::{parse_aac_timescale, parse_avc_timescale, parse_mp3_timescale, parse_timescale, AudioParseResult, Avc1ParseResult, KeyframeType, Parser, VideoParseResult};
use crate::fmpeg::remux_context::{RemuxContext, SampleContextBuilder, TrackContext, TrackType};
use std::cmp::PartialEq;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::thread::JoinHandle;

pub struct Remuxer {
    channel_exchange: Option<mpsc::Sender<Packed>>,
    channel_receiver: mpsc::Receiver<PackedContent>,
    channel_sender: mpsc::Sender<PackedContent>,
    remuxing: bool,

    tags: VecDeque<Tag>,
    metadata: Option<RawMetaData>,
    flv_header: Option<FlvHeader>,

    ctx: RemuxContext,

    audio_track: TrackContext,
    video_track: TrackContext,

    _temp: Option<Vec<u8>>
}

impl ExchangeRegistrable for Remuxer {
    fn set_exchange(&mut self, sender: mpsc::Sender<Packed>) {
        self.channel_exchange = Some(sender);
    }

    fn get_sender(&self) -> mpsc::Sender<PackedContent> {
        self.channel_sender.clone()
    }

    fn get_self_as_destination(&self) -> Destination {
        Destination::Remuxer
    }
}

impl PartialEq for KeyframeType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (KeyframeType::Keyframe, KeyframeType::Keyframe) => true,
            (KeyframeType::Interframe, KeyframeType::Interframe) => true,
            _ => false
        }
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


            audio_track: TrackContext::new(DEFAULT_AUDIO_TRACK_ID, TrackType::Audio),
            video_track: TrackContext::new(DEFAULT_VIDEO_TRACK_ID, TrackType::Video),

            _temp: None
        }
    }

    #[inline]
    fn set_remuxing(&mut self, flag: bool) {
        self.remuxing = flag;
    }

    fn send(&mut self, pack: Packed) -> Result<(), Box<dyn std::error::Error>> {
        match self.channel_exchange.as_ref().unwrap().send(pack) {
            Ok(_) => Ok(()),
            Err(e) => Err("[Remuxer] Channel closed.".into())
        }
    }

    fn send_mpeg4_header(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut header = Encoder::encode_ftyp(&self.ctx).serialize();
        header.append(&mut Encoder::encode_moov(&self.ctx).serialize());
        self.ctx.set_header_sent(true);

        self.send(
            Packed {
                packed_routing: Destination::Core,
                packed_content: PackedContent::ToCore(Data(RemuxedData::Header(header))),
            }
        )
    }

    fn send_raw_data(&mut self, data: RemuxedData) -> Result<(), Box<dyn std::error::Error>> {
        self.send(
            Packed {
                packed_routing: Destination::Core,
                packed_content: PackedContent::ToCore(Data(data)),
            }
        )
    }

    fn remux(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.ctx.is_configured() && !self.ctx.is_header_sent() {
            self.send_mpeg4_header()?;
            if let Some(tmp) = self._temp.take() {
                self.send_raw_data(RemuxedData::Audio(tmp))?;
            }
        }

        while let Some(ref tag) = self.tags.pop_front() {
            match tag.tag_type {
                TagType::Audio => {
                    let parsed = Parser::parse_audio(tag)?;
                    if self.ctx.is_configured() {
                        if !self.ctx.is_header_sent() {
                            self.send_mpeg4_header()?;
                            if let Some(tmp) = self._temp.take() {
                                self.send_raw_data(RemuxedData::Audio(tmp))?;
                            }
                        }
                        match parsed {
                            AudioParseResult::AacRaw(raw) => {
                                let mut sample_ctx = SampleContextBuilder::new()
                                    .set_decode_time(parse_timescale(tag.timestamp))
                                    .set_sample_size(raw.len() as u32)
                                    .set_sample_duration(parse_aac_timescale(self.ctx.audio_sample_rate))
                                    .set_composition_time_offset(0)
                                    .build();

                                let mut data = Encoder::encode_moof(&mut self.ctx, &mut self.audio_track, &mut sample_ctx).serialize();
                                data.append(&mut Encoder::encode_mdat(Vec::from(raw)).serialize());
                                self.send_raw_data(RemuxedData::Audio(data))?;
                            }
                            AudioParseResult::Mp3(parsed) => {
                                let mut sample_ctx = SampleContextBuilder::new()
                                    .set_decode_time(parse_timescale(tag.timestamp))
                                    .set_sample_size(parsed.body.len() as u32)
                                    .set_sample_duration(parse_mp3_timescale(parsed.sample_rate, parsed.version))
                                    .set_composition_time_offset(0)
                                    .build();

                                let mut data = Encoder::encode_moof(&mut self.ctx, &mut self.audio_track, &mut sample_ctx).serialize();
                                data.append(&mut Encoder::encode_mdat(parsed.body).serialize());
                                self.send_raw_data(RemuxedData::Audio(data))?;
                            }
                            _ => {
                                panic!("[Remuxer] Aac format header not set!")
                            }
                        }
                    } else {
                        let audio_codec_conf =  self.ctx.configure_audio_metadata(&parsed);

                        if let AudioParseResult::Mp3(parsed) = parsed {
                            let mut sample_ctx = SampleContextBuilder::new()
                                .set_decode_time(parse_timescale(tag.timestamp))
                                .set_sample_size(parsed.body.len() as u32)
                                .set_sample_duration(parse_mp3_timescale(parsed.sample_rate, parsed.version))
                                .set_composition_time_offset(0)
                                .build();

                            let mut data = Encoder::encode_moof(&mut self.ctx, &mut self.audio_track, &mut sample_ctx).serialize();
                            data.append(&mut Encoder::encode_mdat(parsed.body).serialize());
                            self._temp = Some(data);
                        }

                        if let Some(conf) = audio_codec_conf {
                            self.send( Packed {
                                packed_routing: Destination::Core,
                                packed_content: PackedContent::ToCore(
                                    PackedContentToCore::DecoderConfig(
                                        MseDecoderConfig::AudioCodec(conf)
                                    )
                                )
                            })?;
                        }
                    }
                }
                TagType::Video => {
                    let parsed = Parser::parse_video(tag)?;
                    if self.ctx.is_configured() {
                        if !self.ctx.is_header_sent() {
                            self.send_mpeg4_header()?;
                            if let Some(tmp) = self._temp.take() {
                                self.send_raw_data(RemuxedData::Video(tmp))?;
                            }
                        }
                        if let VideoParseResult::Avc1(parsed) = parsed {
                            match parsed {
                                Avc1ParseResult::AvcNalu(data) => {
                                    let mut sample_ctx = SampleContextBuilder::new()
                                        .set_decode_time(parse_timescale(tag.timestamp))
                                        .set_sample_size(data.payload.len() as u32)
                                        .set_sample_duration(parse_avc_timescale(self.ctx.fps as f32))
                                        .set_composition_time_offset(0)
                                        .set_has_redundancy(false)
                                        .set_is_leading(self.video_track.sequence_number == 1)
                                        .set_is_keyframe(data.keyframe_type == KeyframeType::Keyframe)
                                        .set_is_non_sync(data.keyframe_type == KeyframeType::Interframe)
                                        .build();

                                    let mut send_data = Encoder::encode_moof(&mut self.ctx, &mut self.video_track, &mut sample_ctx).serialize();
                                    send_data.append(&mut Encoder::encode_mdat(Vec::from(data.payload)).serialize());
                                    self.send_raw_data(RemuxedData::Video(send_data))?;
                                }
                                Avc1ParseResult::AvcSequenceHeader(_) => {
                                    panic!("[Remuxer] Avc sequence header not set!")
                                }
                                Avc1ParseResult::AvcEndOfSequence => {
                                    // todo: handle end of sequence
                                    println!("[Remuxer] End of sequence.")
                                }
                            }
                        }
                    } else {
                        // todo: make this method return video codec configuration.
                        // todo: for ctx.configure_audio_metadata(), do the same thing.
                        if let Some(conf) = self.ctx.configure_video_metadata(&parsed) {
                            self.send(
                                Packed {
                                    packed_routing: Destination::Core,
                                    packed_content: PackedContent::ToCore(
                                        PackedContentToCore::DecoderConfig(
                                            MseDecoderConfig::VideoCodec(conf)
                                        )
                                    )
                                }
                            )?;
                        }
                    }
                }
                TagType::Script => {}
                TagType::Encryption => {}
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Ok(received) = self.channel_receiver.recv() {
                if let PackedContent::ToRemuxer(content) = received {
                    match content {
                        PackedContentToRemuxer::PushTag(tag) => {
                            // println!("Pushed tag.");
                            self.tags.push_back(tag);
                        }
                        PackedContentToRemuxer::PushFlvHeader(flv_header) => {
                            println!("[Remuxer] Pushed flv header.");
                            self.ctx.parse_flv_header(&flv_header);
                            self.flv_header = Some(flv_header);
                        }
                        PackedContentToRemuxer::PushMetadata(metadata) => {
                            println!("[Remuxer] Pushed metadata.");
                            self.ctx.parse_metadata(&metadata);
                            self.metadata = Some(metadata);
                        }
                        PackedContentToRemuxer::StartRemuxing => {
                            println!("[Remuxer] Start remuxing.");
                            self.set_remuxing(true)
                        }
                        PackedContentToRemuxer::StopRemuxing => {
                            println!("[Remuxer] Stop remuxing.");
                            self.set_remuxing(false)
                        }
                        PackedContentToRemuxer::CloseWorkerThread => {
                            println!("[Remuxer] Closing remuxer thread.");
                            return Ok(());
                        }
                        PackedContentToRemuxer::Now => { }
                    }
                }
            } else {
                println!("[Remuxer] Channel closed.");
                break;
            }

            if !self.remuxing {
                continue;
            }

            if self.ctx.is_metadata_complete() {
                if self.remux().is_err() {
                    println!("[Remuxer] Channel closed");
                    break;
                }
            } else {
                println!("[Remuxer] Not configured yet.");
            }
        }
        Ok(())
    }

    pub fn launch_worker_thread(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            if let Err(e) = self.run() {
                panic!("Remuxer worker thread stopped unexpectedly: {}", e)
            }
        })
    }
}