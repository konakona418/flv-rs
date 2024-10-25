use crate::flv::header::FlvHeader;
use crate::flv::meta::RawMetaData;

pub const TIME_SCALE: u32 = 1000;

pub struct RemuxContext {
    pub fps: f64,
    pub fps_num: u32,

    pub duration_ms: u32,

    pub width: f64,
    pub height: f64,

    pub has_audio: bool,
    pub has_video: bool,

    pub audio_codec_id: u8,
    pub audio_data_rate: u32,
    pub audio_sample_rate: u32,
    pub audio_channels: u8,

    pub video_codec_id: u8,
    pub video_data_rate: u32,

    pub major_brand: String,
    pub minor_version: String,
    pub compatible_brands: Vec<String>,

    pub video_codec_type: VideoCodecType,
    pub audio_codec_type: AudioCodecType,

    header_sent: bool,
    flv_header_configured: bool,
    metadata_configured: bool,
    video_metadata_configured: bool,
    audio_metadata_configured: bool,
}

pub enum VideoCodecType {
    Avc1,
    None
}

pub enum AudioCodecType {
    Aac,
    Mp3,
    None,
}

impl RemuxContext {
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            fps_num: 0,
            duration_ms: 0,

            width: 0.0,
            height: 0.0,

            has_audio: false,
            has_video: false,

            audio_codec_id: 0,
            audio_data_rate: 0,
            audio_sample_rate: 0,
            audio_channels: 0,

            video_codec_id: 0,
            video_data_rate: 0,

            major_brand: String::from("isom"),
            minor_version: String::from("512"),
            compatible_brands: vec![String::from("isom"), String::from("iso2"), String::from("avc1"), String::from("mp41")],

            video_codec_type: VideoCodecType::None,
            audio_codec_type: AudioCodecType::None,

            header_sent: false,
            flv_header_configured: false,
            metadata_configured: false,
            video_metadata_configured: false,
            audio_metadata_configured: false,
        }
    }

    pub fn parse_flv_header(&mut self, header: &FlvHeader) {
        self.has_audio = header.type_flags_audio;
        self.has_video = header.type_flags_video;
        self.flv_header_configured = true;
    }

    pub fn parse_metadata(&mut self, metadata: &RawMetaData) {
        if let Some(duration) = metadata.try_get_number("duration") {
            self.duration_ms = (duration * TIME_SCALE as f64) as u32;
        }

        if let Some(width) = metadata.try_get_number("width") {
            self.width = width;
        }

        if let Some(height) = metadata.try_get_number("height") {
            self.height = height;
        }

        if let Some(frame_rate) = metadata.try_get_number("framerate") {
            self.fps = frame_rate;
            self.fps_num = (frame_rate * TIME_SCALE as f64) as u32;
        }

        if let Some(audio_codec_id) = metadata.try_get_number("audiocodecid") {
            self.audio_codec_id = audio_codec_id as u8;
        }

        if let Some(audio_data_rate) = metadata.try_get_number("audiodatarate") {
            self.audio_data_rate = audio_data_rate as u32;
        }

        if let Some(video_codec_id) = metadata.try_get_number("videocodecid") {
            self.video_codec_id = video_codec_id as u8;
        }

        if let Some(video_data_rate) = metadata.try_get_number("videodatarate") {
            self.video_data_rate = video_data_rate as u32;
        }

        if let Some(major_brand) = metadata.try_get_string("major_brand") {
            self.major_brand = major_brand;
        }

        if let Some(minor_version) = metadata.try_get_string("minor_version") {
            self.minor_version = minor_version;
        }

        if let Some(mut compatible_brands) = metadata.try_get_string("compatible_brands") {
            self.compatible_brands.push(String::from_iter(compatible_brands.drain(0..4)));
            self.compatible_brands.push(String::from_iter(compatible_brands.drain(0..4)));
            self.compatible_brands.push(String::from_iter(compatible_brands.drain(0..4)));
            self.compatible_brands.push(String::from_iter(compatible_brands.drain(0..4)));
        }

        self.metadata_configured = true;
    }

    pub fn configure_video_metadata(&mut self) {
        self.video_metadata_configured = true;
         todo!()
    }

    pub fn configure_audio_metadata(&mut self) {
        self.audio_metadata_configured = true;
        todo!()
    }

    pub fn is_configured(&self) -> bool {
        self.flv_header_configured && self.metadata_configured && self.video_metadata_configured && self.audio_metadata_configured
    }

    /// for testing only!!
    pub fn _set_configured(&mut self, flag: bool) {
        self.metadata_configured = flag;
        self.flv_header_configured = flag;
        self.video_metadata_configured = flag;
        self.audio_metadata_configured = flag;
    }

    pub fn header_sent(&self) -> bool {
        self.header_sent
    }

    pub fn set_header_sent(&mut self, flag: bool) {
        self.header_sent = flag;
    }
}