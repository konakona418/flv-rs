use crate::fmpeg::mp4frag::{MovieDataBox, MovieFragmentBox, SampleDependencyTableBoxBuilder, SampleFlagBuilder, TrackFragmentBox, TrackFragmentBoxBuilder, TrackRunBoxBuilder};
use crate::fmpeg::mp4head;
use crate::fmpeg::mp4head::aac_utils::AacAudioSpecConfLike;
use crate::fmpeg::mp4head::{AudioMediaHandlerBox, FileTypeBox, FixedPoint32, HandlerType, MediaBox, MovieBox, MovieHeaderBox, SampleBoxTableBox, VideoMediaHandlerBox, XMediaHandlerBox};
use crate::fmpeg::remux_context::{AudioCodecType, SampleContext, RemuxContext, TrackContext, TrackType, VideoCodecType, TIME_SCALE};

pub struct Encoder;

pub const DEFAULT_VIDEO_TRACK_ID: u32 = 1;
pub const DEFAULT_AUDIO_TRACK_ID: u32 = 2;

impl Encoder {
    pub fn encode_ftyp(ctx: &RemuxContext) -> FileTypeBox {
        let ftyp = mp4head::FileTypeBoxBuilder::new()
            .major_brand(&ctx.major_brand)
            .minor_version(ctx.minor_version.parse().unwrap())
            .compatible_brands(ctx.compatible_brands.clone())
            .build();
        dbg!(&ftyp);
        ftyp
    }

    pub fn encode_moov(ctx: &RemuxContext) -> MovieBox {
        let moov = mp4head::MovieBoxBuilder::new()
            .movie_header_box(Self::encode_mhdv(ctx))
            .track(Self::encode_trak(ctx, DEFAULT_VIDEO_TRACK_ID, Self::encode_mdia(ctx, HandlerType::Video)))
            .track(Self::encode_trak(ctx, DEFAULT_AUDIO_TRACK_ID, Self::encode_mdia(ctx, HandlerType::Audio)))
            .build();
        dbg!(&moov);
        moov
    }

    pub fn encode_mhdv(ctx: &RemuxContext) -> MovieHeaderBox {
        let mhdv = mp4head::MovieHeaderBoxV0Builder::new()
            .creation_time(0)
            .modification_time(0)
            .duration(ctx.duration_ms)
            .timescale(TIME_SCALE)
            .next_track_id(3)
            .rate(1.0)
            .volume(1.0)
            .build();
        dbg!(&mhdv);
        MovieHeaderBox::V0(mhdv)
    }

    pub fn encode_trak(ctx: &RemuxContext, track_id: u32, media_box: MediaBox) -> mp4head::TrackBox {
        let trak = mp4head::TrackBox::new(
            mp4head::TrackHeaderBox::V0(
                mp4head::TrackHeaderBoxV0Builder::new()
                    .track_id(track_id)
                    .duration(ctx.duration_ms)
                    .creation_time(0)
                    .modification_time(0)
                    .width(FixedPoint32::from(ctx.width))
                    .height(FixedPoint32::from(ctx.height))
                    .build()
            ),
            media_box
        );
        dbg!(&trak);
        trak
    }

    pub fn encode_mdia(ctx: &RemuxContext, handler_type: HandlerType) -> MediaBox {
        let mdia = mp4head::MediaBox::new(
            Self::encode_mdhd(ctx),
            Self::encode_hdlr(ctx, handler_type.clone()),
            Self::encode_minf(ctx, handler_type),
        );
        dbg!(&mdia);
        mdia
    }

    pub fn encode_mdhd(ctx: &RemuxContext) -> mp4head::MediaHeaderBoxV0 {
        let mdhd = mp4head::MediaHeaderBoxV0Builder::new()
            .creation_time(0)
            .modification_time(0)
            .timescale(TIME_SCALE)
            .duration(ctx.duration_ms)
            .build();
        dbg!(&mdhd);
        mdhd
    }

    pub fn encode_hdlr(ctx: &RemuxContext, handler_type: HandlerType) -> mp4head::HandlerBox {
        let hdlr = handler_type.into();
        dbg!(&hdlr);
        hdlr
    }

    pub fn encode_minf(ctx: &RemuxContext, handler_type: HandlerType) -> mp4head::MediaInfoBox {
        let xmhd: XMediaHandlerBox = match handler_type {
            HandlerType::Video => {
                XMediaHandlerBox::Video(VideoMediaHandlerBox::new())
            }
            HandlerType::Audio => {
                XMediaHandlerBox::Audio(AudioMediaHandlerBox::new())
            }
        };
        let dinf = mp4head::DataInformationBox::new();
        let stsd = mp4head::SampleDescriptionTableBoxBuilder::new()
            .add_sample_description_table_box(
                match handler_type {
                    HandlerType::Video => {
                        if let VideoCodecType::Avc1 = ctx.video_codec_type {
                            mp4head::SubSampleDescriptionTableBox::Avc1(
                                mp4head::Avc1DescriptionBoxBuilder::new()
                                    .set_width(ctx.width as u16)
                                    .set_height(ctx.height as u16)
                                    .avcc_box(ctx.video_avcc_info.clone())
                                    // todo: here is the place to add video configuration
                                    .build()
                            )
                        } else {
                            panic!("Unsupported video codec type")
                        }
                    }
                    HandlerType::Audio => {
                        match ctx.audio_codec_type {
                            AudioCodecType::Aac => {
                                mp4head::SubSampleDescriptionTableBox::Mp4a(
                                    mp4head::Mp4aDescriptionBoxBuilder::new()
                                        .sample_rate(ctx.audio_sample_rate as f32)
                                        .num_audio_channels(ctx.audio_channels as u16)
                                        .spec_config(AacAudioSpecConfLike::VectorConfig(ctx.audio_aac_info.clone()))
                                        .build()
                                )
                            }
                            AudioCodecType::Mp3 => {
                                mp4head::SubSampleDescriptionTableBox::Mp3(
                                    mp4head::Mp3DescriptionBoxBuilder::new()
                                        .sample_rate(ctx.audio_sample_rate as f32)
                                        .num_audio_channels(ctx.audio_channels as u16)
                                        .build()
                                )
                            }
                            AudioCodecType::None => {
                                panic!("Unsupported audio codec type")
                            }
                        }
                    }
                }
            )
            .build();
        let stbl = SampleBoxTableBox::new(stsd);
        let minf = mp4head::MediaInfoBox::new(
            xmhd,
            dinf,
            stbl,
        );
        dbg!(&minf);
        minf
    }

    // todo: implement moof & mdat encoding.

    pub fn encode_moof(ctx: &RemuxContext, track_ctx: &mut TrackContext, encoding_ctx: &SampleContext) -> MovieFragmentBox {
        let mut moof = MovieFragmentBox::new(
            track_ctx.sequence_number,
            Self::encode_traf(ctx, track_ctx, encoding_ctx),
        );
        moof.deferred_set_trun_size();
        moof
    }

    fn encode_traf(ctx: &RemuxContext, track_ctx: &mut TrackContext, encoding_ctx: &SampleContext) -> TrackFragmentBox {
        let traf = TrackFragmentBoxBuilder::new()
            .with_track_id(track_ctx.track_id)
            .with_media_decode_time(encoding_ctx.decode_time) // dts
            .with_sample_table_box(
                match track_ctx.track_type {
                    TrackType::Video => {
                        if encoding_ctx.is_keyframe {
                            SampleDependencyTableBoxBuilder::VideoKeyFrame
                        } else {
                            SampleDependencyTableBoxBuilder::VideoInterFrame
                        }
                    },
                    TrackType::Audio => SampleDependencyTableBoxBuilder::Audio,
                }
            )
            .with_track_run_box(
                TrackRunBoxBuilder::new()
                    .with_data_offset(0)
                    .with_sample_composition_time_offset(encoding_ctx.composition_time_offset) // pts-dts
                    .with_sample_size(encoding_ctx.sample_size)
                    .with_sample_duration(encoding_ctx.sample_duration)
                    .with_sample_flags(
                        SampleFlagBuilder::new()
                            .set_is_leading(encoding_ctx.is_leading)
                            .set_is_non_sync(encoding_ctx.is_non_sync)
                            .set_sample_has_redundancy(encoding_ctx.has_redundancy)
                            .set_sample_depends_on(!encoding_ctx.is_keyframe)
                            .set_sample_is_depended_on(encoding_ctx.is_keyframe)
                            .build()
                    )
                    .build()
            )
            .build();
        dbg!(&traf);

        // note: sequence number has already been increased HERE.
        track_ctx.sequence_number += 1;
        traf
    }

    pub fn encode_mdat(raw_data: Vec<u8>) -> MovieDataBox {
        let mdat = MovieDataBox::new(raw_data);
        mdat
    }
}