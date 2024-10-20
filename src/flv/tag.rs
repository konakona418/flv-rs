use crate::flv::header::TagHeader;

pub struct Tag {
    pub filter: bool,
    pub tag_type: u8,
    pub data_size: u32,
    pub timestamp: u32,
    pub timestamp_extended: u8,
    pub stream_id: u32,
    pub tag_header: TagHeader
}

impl Tag {
    pub fn new(filter: bool, tag_type: u8, data_size: u32, timestamp: u32, timestamp_extended: u8, stream_id: u32, tag_header: TagHeader) -> Self {
        Self { filter, tag_type, data_size, timestamp, timestamp_extended, stream_id, tag_header }
    }
}