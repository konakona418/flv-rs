use std::collections::HashMap;
use crate::flv::script::{ScriptData, ScriptTagBody};

pub struct MetaData {
    pub audio_codec_id: f64,
    pub audio_data_rate: f64,
    pub audio_delay: f64,
    pub audio_sample_rate: f64,
    pub audio_samples_size: f64,
    pub can_seek_to_end: bool,
    pub creation_date: String,
    pub duration: f64,
    pub file_size: f64,
    pub frame_rate: f64,
    pub height: f64,
    pub stereo: bool,
    pub video_codec_id: f64,
    pub video_data_rate: f64,
    pub width: f64,
}

impl MetaData {
    pub fn new(
        audio_codec_id: f64,
        audio_data_rate: f64,
        audio_delay: f64,
        audio_sample_rate: f64,
        audio_samples_size: f64,
        can_seek_to_end: bool,
        creation_date: String,
        duration: f64,
        file_size: f64,
        frame_rate: f64,
        height: f64,
        stereo: bool,
        video_codec_id: f64,
        video_data_rate: f64,
        width: f64,
    ) -> Self {
        Self {
            audio_codec_id,
            audio_data_rate,
            audio_delay,
            audio_sample_rate,
            audio_samples_size,
            can_seek_to_end,
            creation_date,
            duration,
            file_size,
            frame_rate,
            height,
            stereo,
            video_codec_id,
            video_data_rate,
            width,
        }
    }
}

pub struct RawMetaData {
    data: HashMap<String, ScriptData>
}

impl RawMetaData {
    pub fn new(data: ScriptTagBody) -> Self {
        let map = data.value.properties.into_iter().map(|pair| (pair.name.data, pair.value)).collect();
        Self { data: map }
    }

    #[inline]
    pub fn try_get(&self, key: &str) -> Option<ScriptData> {
        self.data.get(key).cloned()
    }

    pub fn try_get_string(&self, key: &str) -> Option<String> {
        match self.try_get(key) {
            Some(ScriptData::String(string)) => Some(string.data),
            Some(ScriptData::LongString(string)) => Some(string.data),
            _ => None,
        }
    }

    pub fn try_get_number(&self, key: &str) -> Option<f64> {
        match self.try_get(key) {
            Some(ScriptData::Number(number)) => Some(number),
            _ => None,
        }
    }

    pub fn try_get_boolean(&self, key: &str) -> Option<bool> {
        match self.try_get(key) {
            Some(ScriptData::Boolean(boolean)) => Some(boolean == 1),
            _ => None,
        }
    }
}

pub struct XMPData {
    pub xmp: String,
}

impl XMPData {
    pub fn new(xmp: String) -> Self {
        Self { xmp }
    }
}