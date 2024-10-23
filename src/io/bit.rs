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