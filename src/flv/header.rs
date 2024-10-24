use crate::flv::decoder::Decoder;
use crate::io::bit::BitIO;

#[derive(Debug)]
pub struct FlvHeader {
    pub signature: [u8; 3],
    pub version: u8,
    pub type_flags_audio: bool,
    pub type_flags_video: bool,
    pub data_offset: u32,
}

impl FlvHeader {
    pub fn new(signature: [u8; 3], version: u8, type_flags_audio: bool, type_flags_video: bool, data_offset: u32) -> Self {
        Self { signature, version, type_flags_audio, type_flags_video, data_offset }
    }
}

#[derive(Debug, Clone)]
pub enum TagHeader {
    Audio(AudioTagHeader),
    Video(VideoTagHeader),
    Script,
    Placeholder,
}

#[derive(Debug, Clone)]
pub struct AudioTagHeader {
    // UB4
    pub sound_format: u8,
    // UB2
    pub sound_rate: u8,
    // UB1
    pub sound_size: bool,
    // UB1
    pub sound_type: bool,
    // UI8
    // if sound_format == 10
    pub aac_packet_type: Option<u8>,
}

impl AudioTagHeader {
    pub fn new(sound_format: u8, sound_rate: u8, sound_size: bool, sound_type: bool, aac_packet_type: Option<u8>) -> Self {
        Self { sound_format, sound_rate, sound_size, sound_type, aac_packet_type }
    }

    pub fn parse(decoder: &mut Decoder, header_size: &mut usize) -> Result<Self, Box<dyn std::error::Error>> {
        *header_size += 1;
        let bits = BitIO::new(decoder.drain_u8());
        let sound_format = bits.read_range(0, 3);
        let sound_rate = bits.read_range(4, 5);
        let sound_size = bits.read_bit(6);
        let sound_type = bits.read_bit(7);

        let aac_packet_type = if sound_format == 10 {
            *header_size += 1;
            Some(decoder.drain_u8())
        } else {
            None
        };
        Ok(Self { sound_format, sound_rate, sound_size, sound_type, aac_packet_type })
    }
}

#[derive(Debug, Clone)]
pub struct VideoTagHeader {
    // UB4
    pub frame_type: u8,
    // UB4
    pub codec_id: u8,
    // UI24
    // if codec_id == 7
    pub avc_packet_type: Option<u8>,
    // SI24
    // if codec_id == 7
    pub composition_time: Option<i32>,
}

impl VideoTagHeader {
    pub fn new(frame_type: u8, codec_id: u8, avc_packet_type: Option<u8>, composition_time: Option<i32>) -> Self {
        Self { frame_type, codec_id, avc_packet_type, composition_time }
    }

    pub fn parse(decoder: &mut Decoder, header_size: &mut usize) -> Result<Self, Box<dyn std::error::Error>> {
        *header_size += 1;
        let bits = BitIO::new(decoder.drain_u8());
        let frame_type = bits.read_range(0, 3);
        let codec_id = bits.read_range(4, 7);

        let mut avc_packet_type = None;
        let mut composition_time = None;
        if codec_id == 7 {
            *header_size += 1;
            avc_packet_type = Some(decoder.drain_u8());

            *header_size += 3;
            composition_time = Some(decoder.drain_i24());
        }
        Ok(Self { frame_type, codec_id, avc_packet_type, composition_time })
    }
}

#[derive(Debug, Clone)]
pub struct EncryptionTagHeader {
    // todo: encryption
}

impl EncryptionTagHeader {
    pub fn parse(decoder: &mut Decoder, header_size: &mut usize) -> Result<Self, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub enum FilterParameters {
    EncryptionFilter(EncryptionFilterParameters),
    SelectiveEncryptionFilter(SelectiveEncryptionFilterParameters),
}

#[derive(Debug, Clone)]
pub struct EncryptionFilterParameters {
    // todo: encryption
}

#[derive(Debug, Clone)]
pub struct SelectiveEncryptionFilterParameters {
    // todo: selective encryption
}

impl SelectiveEncryptionFilterParameters {
    pub fn parse(decoder: &mut Decoder) -> Self {
        unimplemented!()
    }
}

impl FilterParameters {
    pub fn parse(decoder: &mut Decoder, param_size: &mut usize) -> Result<Self, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}