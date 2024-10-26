pub struct BitIO {
    byte: u8
}

impl BitIO {
    pub fn new(byte: u8) -> BitIO {
        BitIO {
            byte
        }
    }

    #[inline]
    pub fn read(&self) -> bool {
        (self.byte & 1) != 0
    }

    /// Note: the idx param is counted from the left.
    /// This is the same in the read_range method.
    #[inline]
    pub fn read_bit(&self, index: usize) -> bool {
        (self.byte & (1 << (7 - index))) != 0
    }

    #[inline]
    pub fn read_bit_safe(&self, index: usize) -> Result<bool, Box<dyn std::error::Error>> {
        if index > 7 {
            Err("Index out of range".into())
        } else {
            Ok((self.byte & (1 << index)) != 0)
        }
    }

    /// Note: the start and end params are counted from the left,
    /// just like read_bit method.
    /// Why? because it's easier to read.
    #[inline]
    pub fn read_range(&self, start: usize, end: usize) -> u8 {
        let mut mask: u8 = 0b11111111u8;
        mask >>= start;
        mask <<= 7 - end;
        (self.byte & mask) >> (7 - end)
    }
}

pub enum UIntParserEndian {
    LittleEndian,
    BigEndian
}
pub struct U16BitIO {
    pub(crate) data: [u8; 2],
    endian: UIntParserEndian
}

impl U16BitIO {
    #[inline]
    pub fn get_data(&self) -> [u8; 2] {
        self.data
    }

    #[inline]
    pub fn new(data: u16, endian: UIntParserEndian) -> U16BitIO {
        let data = match endian {
            UIntParserEndian::LittleEndian => {
                data.to_le_bytes()
            }
            UIntParserEndian::BigEndian => {
                data.to_be_bytes()
            }
        };
        Self {
            data,
            endian
        }
    }

    /// offset is counted from the left of u16.
    #[inline]
    pub fn read_at(&mut self, bit_offset: usize) -> bool {
        let byte_offset = bit_offset / 8;
        let bit_offset = bit_offset % 8;

        let byte = self.data[byte_offset];
        let mask = 1 << (7 - bit_offset);
        byte & mask != 0
    }

    /// read a range of bits.
    /// contains both start and end inclusive.
    #[inline]
    pub fn read_range(&mut self, start: usize, end: usize) -> u16 {
        let mut result = 0;
        for i in start..end + 1 {
            result |= (self.read_at(i) as u16) << (end - i);
        }
        result
    }

    #[inline]
    pub fn write_at(&mut self, bit_offset: usize, value: bool) {
        let byte_offset = bit_offset / 8;
        let bit_offset = bit_offset % 8;
        let mask = 1 << (7 - bit_offset);
        if value {
            self.data[byte_offset] |= mask;
        } else {
            self.data[byte_offset] &= !mask;
        }
    }

    #[inline]
    pub fn write_range(&mut self, start: usize, end: usize, value: u16) {
        for i in start..end + 1 {
            self.write_at(i, (value & (1 << (end - i))) != 0);
        }
    }
}

pub struct U32BitIO {
    data: [u8; 4],
    endian: UIntParserEndian
}

impl U32BitIO {
    #[inline]
    pub fn get_data(&self) -> [u8; 4] {
        self.data
    }

    #[inline]
    pub fn new(data: u32, endian: UIntParserEndian) -> U32BitIO {
        let data = match endian {
            UIntParserEndian::LittleEndian => {
                data.to_le_bytes()
            }
            UIntParserEndian::BigEndian => {
                data.to_be_bytes()
            }
        };
        Self {
            data,
            endian
        }
    }

    #[inline]
    pub fn read_at(&mut self, bit_offset: usize) -> bool {
        let byte_offset = bit_offset / 8;
        let bit_offset = bit_offset % 8;

        let byte = self.data[byte_offset];
        let mask = 1 << (7 - bit_offset);
        byte & mask != 0
    }

    #[inline]
    pub fn read_range(&mut self, start: usize, end: usize) -> u32 {
        let mut result = 0;
        for i in start..end + 1 {
            result |= (self.read_at(i) as u32) << (end - i);
        }
        result
    }

    #[inline]
    pub fn write_at(&mut self, bit_offset: usize, value: bool) {
        let byte_offset = bit_offset / 8;
        let bit_offset = bit_offset % 8;
        let mask = 1 << (7 - bit_offset);
        if value {
            self.data[byte_offset] |= mask;
        } else {
            self.data[byte_offset] &= !mask;
        }
    }

    #[inline]
    pub fn write_range(&mut self, start: usize, end: usize, value: u32) {
        for i in start..end + 1 {
            self.write_at(i, (value & (1 << (end - i))) != 0);
        }
    }
}