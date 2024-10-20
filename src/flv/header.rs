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

pub enum TagHeader {
    Audio(AudioTagHeader),
    Video(VideoTagHeader),
    Encryption(EncryptionTagHeader),
    Script,
}

pub struct AudioTagHeader {
    // UB4
    pub sound_format: u8,
    // UB2
    pub sound_rate: u8,
    // UB1
    pub sound_size: bool,
    // UB1
    pub sound_type: u8,
    // UI8
    // if sound_format == 10
    pub aac_packet_type: u8,
}

impl AudioTagHeader {
    pub fn new(sound_format: u8, sound_rate: u8, sound_size: bool, sound_type: u8, aac_packet_type: u8) -> Self {
        Self { sound_format, sound_rate, sound_size, sound_type, aac_packet_type }
    }
}

pub struct VideoTagHeader {
    // UB4
    pub frame_type: u8,
    // UB4
    pub codec_id: u8,
    // UI24
    // if codec_id == 7
    pub avc_packet_type: u8,
    // SI24
    // if codec_id == 7
    pub composition_time: i32,
}

impl VideoTagHeader {
    pub fn new(frame_type: u8, codec_id: u8, avc_packet_type: u8, composition_time: i32) -> Self {
        Self { frame_type, codec_id, avc_packet_type, composition_time }
    }
}

pub struct EncryptionTagHeader {
    // todo: encryption
}