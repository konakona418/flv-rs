use crate::flv::header::FlvHeader;
use crate::flv::tag::Tag;
use crate::io::bit::BitIO;

pub struct Decoder {
    data: Vec<u8>,
    previous_tag_size: u32
}

impl Decoder {
    pub fn new(data: Vec<u8>) -> Self {
        Decoder {
            data,
            previous_tag_size: 0
        }
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
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(self.data.remove(0));
        }
        result
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
        let data_offset = self.drain_u32_le();
        Ok(
            FlvHeader::new(
                signature,
                version,
                has_audio,
                has_video,
                data_offset
            )
        )
    }

    pub fn decode_tag(&mut self) -> Result<Tag, Box<dyn std::error::Error>> {
        let bit = BitIO::new(self.drain_u8());
        let filter = bit.read_bit(2);
        let tag_type = bit.read_range(3, 7);
        let data_size = self.drain_u24();
        let timestamp = self.drain_u24();
        let timestamp_extended = self.drain_u8();
        let stream_id = self.drain_u24();
        Ok(())
    }

    pub fn decode_body(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.previous_tag_size != 0 {
            self.previous_tag_size = self.drain_u32_le();
        } else {
            // Skip first previous tag size
            self.drain_u32_le();
        }
        Ok(())
    }
}