mod flv;
mod io;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let byte = 0b10101011;
        let bit_io = io::bit::BitIO::new(byte);
        assert_eq!(bit_io.read(), true);
        assert_eq!(bit_io.read_bit(2), false);
        assert_eq!(bit_io.read_bit(7), true);
        assert_eq!(bit_io.read_bit(6), false);
        assert_eq!(bit_io.read_range(3, 7), 0b10101u8);
        assert_eq!(bit_io.read_range(0, 3), 0b1011u8);
        assert_eq!(bit_io.read_range(3, 5), 0b101u8);
    }
}
