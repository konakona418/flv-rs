use crate::flv::header::{EncryptionTagHeader, FilterParameters, TagHeader};
use crate::flv::script::ScriptTagBody;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub struct Tag {
    pub filter: bool,
    pub tag_type: TagType,
    pub data_size: u32,
    pub timestamp_short: u32,
    pub timestamp_extended: u8,
    pub timestamp: u32,
    pub stream_id: u32,
    pub tag_header: TagHeader,
    pub encryption_tag_header: Option<EncryptionTagHeader>,
    pub filter_parameters: Option<FilterParameters>,
    pub tag_body: TagBody,
}

#[derive(Debug, Clone)]
pub enum TagType {
    Audio,
    Video,
    Script,
    Encryption,
}

impl PartialEq<Self> for TagType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            TagType::Audio => match other {
                TagType::Audio => true,
                _ => false
            },
            TagType::Video => match other {
                TagType::Video => true,
                _ => false
            },
            TagType::Script => match other {
                TagType::Script => true,
                _ => false
            },
            TagType::Encryption => match other {
                TagType::Encryption => true,
                _ => false
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum TagBody {
    Normal(NormalTagBody),
    Encrypted(EncryptedTagBody),
}

#[derive(Clone)]
pub enum NormalTagBody {
    Audio(Vec<u8>),
    Video(Vec<u8>),
    Script(ScriptTagBody),
    Placeholder, // todo: temporary
}

impl Debug for NormalTagBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NormalTagBody::Audio(data) => {
                f.debug_struct("Audio")
                    .field("data_size", &data.len())
                    .finish()
            }
            NormalTagBody::Video(data) => {
                f.debug_struct("Video")
                    .field("data_size", &data.len())
                    .finish()
            }
            NormalTagBody::Script(data) => {
                f.debug_struct("Script")
                    .field("name", &data.name)
                    .field("data_size", &data.value.length)
                    .field("data", &data.value.properties)
                    .finish()
            }
            _ => {
                f.debug_struct("Placeholder")
                    .field("data_size", &0)
                    .finish()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EncryptedTagBody {
    Audio(Vec<u8>),
    Video(Vec<u8>),
    Script(ScriptTagBody),
    Placeholder, // todo: temporary
}

impl TagType {
    pub fn from(tag_type: u8) -> Result<TagType, Box<dyn std::error::Error>> {
        match tag_type {
            8 => {
                Ok(TagType::Audio)
            }
            9 => {
                Ok(TagType::Video)
            }
            18 => {
                Ok(TagType::Script)
            }
            _ => {
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid tag type")))
            }
        }
    }
}

impl Tag {
    pub fn new(
        filter: bool,
        tag_type: TagType,
        data_size: u32,
        timestamp_short: u32,
        timestamp_extended: u8,
        timestamp: u32,
        stream_id: u32,
        tag_header: TagHeader,
        tag_body: TagBody,
        encryption_tag_header: Option<EncryptionTagHeader>,
        filter_parameters: Option<FilterParameters>
    ) -> Self {
        Self {
            filter,
            tag_type,
            data_size,
            timestamp_short,
            timestamp_extended,
            timestamp,
            stream_id,
            tag_header,
            tag_body,
            encryption_tag_header,
            filter_parameters
        }
    }
}