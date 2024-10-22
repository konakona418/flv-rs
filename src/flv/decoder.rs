use crate::core::Core;
use crate::flv::header::{AudioTagHeader, EncryptionTagHeader, FilterParameters, FlvHeader, TagHeader, VideoTagHeader};
use crate::flv::script::ScriptTagBody;
use crate::flv::tag::{EncryptedTagBody, NormalTagBody, Tag, TagBody, TagType};
use crate::io::bit::BitIO;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Decoder {
    data: Vec<u8>,
    previous_tag_size: u32,
    core: Rc<RefCell<Core>>
}

impl Decoder {
    pub fn new(data: Vec<u8>, core: Rc<RefCell<Core>>) -> Self {
        Decoder {
            data,
            previous_tag_size: 0,
            core,
        }
    }

    pub fn push_data(&mut self, data: &mut Vec<u8>) {
        self.data.append(data);
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn drain_u8(&mut self) -> u8 {
        self.data.remove(0)
    }

    pub fn drain_bytes<const SIZE: usize>(&mut self) -> [u8; SIZE] {
        let mut result = [0; SIZE];
        for i in 0..SIZE {
            result[i] = self.data.remove(0);
        }
        result
    }

    pub fn drain_bytes_vec(&mut self, size: usize) -> Vec<u8> {
        let drained = self.data.drain(0..size).collect::<Vec<_>>();
        drained
    }

    pub fn drain_u16_le(&mut self) -> u16 {
        let mut result = 0;
        result |= self.data.remove(0) as u16;
        result |= (self.data.remove(0) as u16) << 8;
        result
    }

    pub fn drain_u16(&mut self) -> u16 {
        let mut result = 0;
        result |= (self.data.remove(0) as u16) << 8;
        result |= self.data.remove(0) as u16;
        result
    }

    pub fn drain_u24_le(&mut self) -> u32 {
        let mut result = 0;
        result |= self.data.remove(0) as u32;
        result |= (self.data.remove(0) as u32) << 8;
        result |= (self.data.remove(0) as u32) << 16;
        result
    }

    pub fn drain_u24(&mut self) -> u32 {
        let mut result = 0;
        result |= (self.data.remove(0) as u32) << 16;
        result |= (self.data.remove(0) as u32) << 8;
        result |= self.data.remove(0) as u32;
        result
    }

    pub fn drain_u32_le(&mut self) -> u32 {
        let mut result = 0;
        result |= self.data.remove(0) as u32;
        result |= (self.data.remove(0) as u32) << 8;
        result |= (self.data.remove(0) as u32) << 16;
        result |= (self.data.remove(0) as u32) << 24;
        result
    }

    pub fn drain_u32(&mut self) -> u32 {
        let mut result = 0;
        result |= (self.data.remove(0) as u32) << 24;
        result |= (self.data.remove(0) as u32) << 16;
        result |= (self.data.remove(0) as u32) << 8;
        result |= self.data.remove(0) as u32;
        result
    }

    pub fn drain_u64(&mut self) -> u64 {
        let mut result = 0;
        result |= (self.data.remove(0) as u64) << 56;
        result |= (self.data.remove(0) as u64) << 48;
        result |= (self.data.remove(0) as u64) << 40;
        result |= (self.data.remove(0) as u64) << 32;
        result |= (self.data.remove(0) as u64) << 24;
        result |= (self.data.remove(0) as u64) << 16;
        result |= (self.data.remove(0) as u64) << 8;
        result |= self.data.remove(0) as u64;
        result
    }

    pub fn drain_i8(&mut self) -> i8 {
        self.data.remove(0) as i8
    }

    pub fn drain_i16(&mut self) -> i16 {
        let mut result = 0;
        result |= (self.data.remove(0) as i16) << 8;
        result |= self.data.remove(0) as i16;
        result
    }

    pub fn drain_i24(&mut self) -> i32 {
        let mut result = 0;
        result |= (self.data.remove(0) as i32) << 16;
        result |= (self.data.remove(0) as i32) << 8;
        result |= self.data.remove(0) as i32;
        result
    }

    pub fn drain_i32(&mut self) -> i32 {
        let mut result = 0;
        result |= (self.data.remove(0) as i32) << 24;
        result |= (self.data.remove(0) as i32) << 16;
        result |= (self.data.remove(0) as i32) << 8;
        result |= self.data.remove(0) as i32;
        result
    }

    pub fn drain_i64(&mut self) -> i64 {
        let mut result = 0;
        result |= (self.data.remove(0) as i64) << 56;
        result |= (self.data.remove(0) as i64) << 48;
        result |= (self.data.remove(0) as i64) << 40;
        result |= (self.data.remove(0) as i64) << 32;
        result |= (self.data.remove(0) as i64) << 24;
        result |= (self.data.remove(0) as i64) << 16;
        result |= (self.data.remove(0) as i64) << 8;
        result |= self.data.remove(0) as i64;
        result
    }

    pub fn drain_f64(&mut self) -> f64 {
        let mut result = 0;
        result |= (self.data.remove(0) as u64) << 56;
        result |= (self.data.remove(0) as u64) << 48;
        result |= (self.data.remove(0) as u64) << 40;
        result |= (self.data.remove(0) as u64) << 32;
        result |= (self.data.remove(0) as u64) << 24;
        result |= (self.data.remove(0) as u64) << 16;
        result |= (self.data.remove(0) as u64) << 8;
        result |= self.data.remove(0) as u64;
        f64::from_bits(result)
    }

    pub fn drain_f64_le(&mut self) -> f64 {
        let mut result = 0;
        result |= self.data.remove(0) as u64;
        result |= (self.data.remove(0) as u64) << 8;
        result |= (self.data.remove(0) as u64) << 16;
        result |= (self.data.remove(0) as u64) << 24;
        result |= (self.data.remove(0) as u64) << 32;
        result |= (self.data.remove(0) as u64) << 40;
        result |= (self.data.remove(0) as u64) << 48;
        result |= (self.data.remove(0) as u64) << 56;
        f64::from_bits(result)
    }

    pub fn drain_f32_le(&mut self) -> f32 {
        let mut result = 0;
        result |= self.data.remove(0) as u32;
        result |= (self.data.remove(0) as u32) << 8;
        result |= (self.data.remove(0) as u32) << 16;
        result |= (self.data.remove(0) as u32) << 24;
        f32::from_bits(result)
    }

    pub fn drain_f32(&mut self) -> f32 {
        let mut result = 0;
        result |= (self.data.remove(0) as u32) << 24;
        result |= (self.data.remove(0) as u32) << 16;
        result |= (self.data.remove(0) as u32) << 8;
        result |= self.data.remove(0) as u32;
        f32::from_bits(result)
    }

    pub fn decode_header(&mut self) -> Result<FlvHeader, Box<dyn std::error::Error>> {
        let signature: [u8; 3] = self.drain_bytes::<3>();
        let version = self.drain_u8();
        let bits = BitIO::new(self.drain_u8());
        let has_audio = bits.read_bit(5);
        let has_video = bits.read_bit(7);
        let data_offset = self.drain_u32();
        Ok(
            FlvHeader::new(
                signature,
                version,
                has_audio,
                has_video,
                data_offset,
            )
        )
    }

    pub fn concat_ts(ts: u32, ts_ext: u8) -> u32 {
        (ts & 0x00FFFFFFu32) | ((ts_ext as u32) << 24)
    }

    pub fn decode_tag(&mut self) -> Result<Tag, Box<dyn std::error::Error>> {
        let bit = BitIO::new(self.drain_u8());
        let filter = bit.read_bit(2);
        let tag_type = TagType::from(bit.read_range(3, 7))?;

        let data_size = self.drain_u24();

        let timestamp = self.drain_u24();
        let timestamp_extended = self.drain_u8();
        let ts_concatenated = Self::concat_ts(timestamp, timestamp_extended);

        let stream_id = self.drain_u24(); // always 0.

        // Note: all the elements before stream_id made up for 11 bytes in total.
        //

        let mut encryption_header = None;
        let mut filter_params = None;

        let mut header_size: usize = 0;

        let tag_header: TagHeader;
        let tag_body = if !filter {
            TagBody::Normal(match tag_type {
                TagType::Audio => {
                    tag_header = TagHeader::Audio(AudioTagHeader::parse(self, &mut header_size)?);
                    // todo: untested
                    NormalTagBody::Audio(self.drain_bytes_vec((data_size as usize) - header_size))
                }
                TagType::Video => {
                    tag_header = TagHeader::Video(VideoTagHeader::parse(self, &mut header_size)?);
                    // todo: untested
                    NormalTagBody::Video(self.drain_bytes_vec((data_size as usize) - header_size))
                }
                TagType::Script => {
                    tag_header = TagHeader::Script;
                    NormalTagBody::Script(ScriptTagBody::parse(self)?)
                }
                _ => {
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid tag type")));
                }
            })
        } else {
            // todo: handle encryption tag header
            // todo: handle filter parameters
            tag_header = TagHeader::Placeholder;
            encryption_header = Some(EncryptionTagHeader::parse(self, &mut header_size)?);
            filter_params = Some(FilterParameters::parse(self, &mut header_size)?);
            TagBody::Encrypted(EncryptedTagBody::Placeholder)
        };

        Ok(Tag::new(
            filter,
            tag_type,
            data_size,
            timestamp,
            timestamp_extended,
            ts_concatenated,
            stream_id,
            tag_header,
            tag_body,
            encryption_header,
            filter_params,
        ))
    }

    pub fn decode_body(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut dbg_cnt = 0;
        loop {
            if self.data.is_empty() || dbg_cnt > 10 {
                break;
            }
            dbg_cnt += 1;
            let previous_tag_size = self.drain_u32();
            dbg!(previous_tag_size);
            if previous_tag_size == self.previous_tag_size {
                let tag = self.decode_tag()?;
                dbg!(tag.data_size + 11);
                self.previous_tag_size = tag.data_size + 11;

                dbg!(tag);
                // todo: send tag to demuxer.
            } else {
                return Err(
                    Box::new(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData, 
                            format!("Tag size mismatch: expected {}, read {}.", previous_tag_size, self.previous_tag_size)
                        )
                    )
                );
            }
        }
        Ok(())
    }
}