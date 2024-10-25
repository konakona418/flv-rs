use std::collections::VecDeque;
use crate::flv::header::{AudioTagHeader, TagHeader};
use crate::flv::tag::{NormalTagBody, Tag, TagBody};
use crate::io;

pub enum AudioParseResult {
    AacRaw(VecDeque<u8>),
    AacSeqHdr(AacSequenceHeader),
    Mp3(Mp3ParseResult)
}

pub enum AudioConfigurationLike {
    Mp3(Mp3ParseResult),
    Aac(AacSequenceHeader),
}

pub enum Mp3Version {
    Mp25,
    Mp20,
    Mp10,
    Reserved
}

impl From<u8> for Mp3Version {
    fn from(value: u8) -> Self {
        match value {
            0 => Mp3Version::Mp25,
            1 => Mp3Version::Reserved,
            2 => Mp3Version::Mp20,
            3 => Mp3Version::Mp10,
            _ => panic!("Invalid mp3 version."),
        }
    }
}

pub enum Mp3Layer {
    Reserved,
    L1,
    L2,
    L3
}

impl From<u8> for Mp3Layer {
    fn from(value: u8) -> Self {
        match value {
            0 => Mp3Layer::Reserved,
            1 => Mp3Layer::L3,
            2 => Mp3Layer::L2,
            3 => Mp3Layer::L1,
            _ => panic!("Invalid mp3 layer."),
        }
    }
}

pub enum Channel {
    Mono,
    Dual,
    Stereo,
    JointStereo
}

impl From<u8> for Channel {
    fn from(value: u8) -> Self {
        match value {
            0 => Channel::Stereo,
            1 => Channel::JointStereo,
            2 => Channel::Dual,
            3 => Channel::Mono,
            _ => panic!("Invalid channel."),
        }
    }
}

pub struct Mp3ParseResult {
    pub version: Mp3Version,
    pub layer: Mp3Layer,
    pub sample_rate: u32,
    pub bitrate: u32,
    pub channel: Channel,
    pub channel_extended: u8,
}

pub const AUDIO_SAMPLE_RATE_TABLE_M10: [u32; 4] = [44100, 48000, 32000, 0];
pub const AUDIO_SAMPLE_RATE_TABLE_M20: [u32; 4] = [22050, 24000, 16000, 0];
pub const AUDIO_SAMPLE_RATE_TABLE_M25: [u32; 4] = [11025, 12000, 8000, 0];

pub const AUDIO_BITRATE_TABLE_L1: [u32; 16] = [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448, 0];
pub const AUDIO_BITRATE_TABLE_L2: [u32; 16] = [0, 32, 48, 56,  64,  80,  96, 112, 128, 160, 192, 224, 256, 320, 384, 0];
pub const AUDIO_BITRATE_TABLE_L3: [u32; 16] = [0, 32, 40, 48,  56,  64,  80,  96, 112, 128, 160, 192, 224, 256, 320, 0];

const MP3_SYNC_WORD: u16 = 0x07FF;

pub struct AacSequenceHeader {
    pub audio_object_type: u8,
    pub sampling_frequency_index: u8,
    pub channel_configuration: u8,
}

pub struct VideoParseResult;

pub struct Parser;

impl Parser {
    pub fn parse_audio(tag: &Tag) -> Result<AudioParseResult, Box<dyn std::error::Error>> {
        let header = match tag.tag_header {
            TagHeader::Audio(ref header) => header,
            _ => return Err("Tag type mismatch.".into()),
        };

        let body = match tag.tag_body {
            TagBody::Normal(ref body) =>
                match body {
                    NormalTagBody::Audio(ref body) => { body }
                    _ => return Err("Tag body type mismatch.".into()),
                },
            _ => return Err("Encrypted audio is not supported.".into()),
        };

        // mp3; aac
        if header.sound_format != 2 && header.sound_format != 10 {
            return Err("Unsupported sound format.".into());
        }

        if header.sound_format == 2 {
            // mp3
            Self::parse_mp3(header, body)
        } else {
            // aac
            Self::parse_aac(header, body)
        }
    }

    fn parse_mp3(header: &AudioTagHeader, body: &VecDeque<u8>) -> Result<AudioParseResult, Box<dyn std::error::Error>> {
        let mut u16io = io::bit::U16BitIO::new(
            <u16>::from_be_bytes(
                [
                    body[0],
                    body[1]
                ]
            ),
            io::bit::UIntParserEndian::BigEndian
        );

        let sync_word = u16io.read_range(0, 11);
        if sync_word != MP3_SYNC_WORD {
            return Err("MP3 sync word mismatch!".into());
        }

        let version = Mp3Version::from(u16io.read_range(12, 13) as u8);
        let layer = Mp3Layer::from(u16io.read_range(14, 15) as u8);
        let protection_bit = u16io.read_at(16);

        let mut u16io = io::bit::U16BitIO::new(
            <u16>::from_be_bytes(
                [
                    body[2],
                    body[3]
                ]
            ),
            io::bit::UIntParserEndian::BigEndian
        );
        let bitrate_index = u16io.read_range(0, 3);
        let sampling_rate_index = u16io.read_range(4, 5);
        // 1 bit padding.
        let _ = u16io.read_range(6, 7);
        let channel_mode = u16io.read_range(8, 9);

        let sample_rate = match version {
            Mp3Version::Mp25 => AUDIO_SAMPLE_RATE_TABLE_M25[sampling_rate_index as usize],
            Mp3Version::Mp20 => AUDIO_SAMPLE_RATE_TABLE_M20[sampling_rate_index as usize],
            Mp3Version::Mp10 => AUDIO_SAMPLE_RATE_TABLE_M10[sampling_rate_index as usize],
            _ => panic!("Invalid mp3 version."),
        };

        let bitrate = match layer {
            Mp3Layer::L1 => AUDIO_BITRATE_TABLE_L1[bitrate_index as usize],
            Mp3Layer::L2 => AUDIO_BITRATE_TABLE_L2[bitrate_index as usize],
            Mp3Layer::L3 => AUDIO_BITRATE_TABLE_L3[bitrate_index as usize],
            _ => panic!("Invalid mp3 layer."),
        };
        // todo: is this okay?

        let channel = Channel::from(channel_mode as u8);
        let channel_extended: u8;
        if let Channel::JointStereo = channel {
            channel_extended = u16io.read_range(10, 11) as u8;
        } else {
            channel_extended = 0;
        }

        Ok(AudioParseResult::Mp3(Mp3ParseResult {
            version,
            layer,
            sample_rate,
            bitrate,
            channel,
            channel_extended,
        }))
    }

    fn parse_aac(header: &AudioTagHeader, body: &VecDeque<u8>) -> Result<AudioParseResult, Box<dyn std::error::Error>> {
        if let Some(aac_pack_type) = header.aac_packet_type {
            match aac_pack_type {
                0 => Self::parse_aac_seq_hdr(body),
                1 => Self::parse_aac_raw(body),
                _ => Err("Unsupported AAC packet type.".into()),
            }
        } else {
            Err("AAC packet type is not set.".into())
        }
    }

    fn parse_aac_seq_hdr(body: &VecDeque<u8>) -> Result<AudioParseResult, Box<dyn std::error::Error>> {
        let mut u16io = io::bit::U16BitIO::new(
            <u16>::from_be_bytes(
                [
                    body[0],
                    body[1]
                ]
            ),
            io::bit::UIntParserEndian::BigEndian
        );
        let audio_object_type = u16io.read_range(0, 4) as u8;
        let sampling_frequency_index = u16io.read_range(5, 8) as u8;
        let channel_configuration = u16io.read_range(9, 12) as u8;
        Ok(AudioParseResult::AacSeqHdr(AacSequenceHeader {
            audio_object_type,
            sampling_frequency_index,
            channel_configuration,
        }))
    }

    fn parse_aac_raw(body: &VecDeque<u8>) -> Result<AudioParseResult, Box<dyn std::error::Error>> {
        Ok(AudioParseResult::AacRaw(body.clone()))
    }

    // todo: implement video parsing.
}