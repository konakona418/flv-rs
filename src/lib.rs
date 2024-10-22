mod flv;
mod io;
mod core;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use crate::flv::decoder::Decoder;
    use super::*;

    #[test]
    fn it_works() {
        let byte = 0b10101011;
        let bit_io = io::bit::BitIO::new(byte);
        assert_eq!(bit_io.read(), true);

        assert_eq!(Decoder::concat_ts(0x123456, 0xAB), 0xAB123456);
        assert_eq!(Decoder::concat_ts(0x00123456, 0xAB), 0xAB123456);
        assert_eq!(Decoder::concat_ts(0x00000000, 0xAB), 0xAB000000);
        assert_eq!(Decoder::concat_ts(0x123456, 0x00), 0x00123456);
        assert_eq!(Decoder::concat_ts(0x00FFFFFF, 0xFF), 0xFFFFFFFF);
        assert_eq!(Decoder::concat_ts(0x00000000, 0x00), 0x00000000);
        assert_eq!(255u8 as i8, -1);

        let mut vec = vec![];
        let mut vec_i16_be = 32767i16.to_be_bytes().to_vec();

        let mut vec_i24_be = 0x00FFFFFFi32.to_be_bytes().to_vec();
        assert_eq!(vec_i24_be.remove(0), 0);
        let mut vec_i24_be = vec_i24_be;

        let mut vec_i32_be = 0x1234abcdi32.to_be_bytes().to_vec();
        let mut vec_i64_be = 0x1234abcd1234abcdi64.to_be_bytes().to_vec();

        let mut vec_f32_be = std::f32::consts::PI.to_be_bytes().to_vec();
        let mut vec_f64_be = std::f64::consts::PI.to_be_bytes().to_vec();

        let mut vec_u16_be = 65535u16.to_be_bytes().to_vec();

        let mut vec_u24_be = 0x00ffffffu32.to_be_bytes().to_vec();
        assert_eq!(vec_u24_be.remove(0), 0);
        let mut vec_u24_be = vec_u24_be;

        let mut vec_u32_be = 4294967295u32.to_be_bytes().to_vec();
        let mut vec_u64_be = 18446744073709551615u64.to_be_bytes().to_vec();

        vec.append(&mut vec_i16_be);
        vec.append(&mut vec_i24_be);
        vec.append(&mut vec_i32_be);
        vec.append(&mut vec_i64_be);

        vec.append(&mut vec_f32_be);
        vec.append(&mut vec_f64_be);

        vec.append(&mut vec_u16_be);
        vec.append(&mut vec_u24_be);
        vec.append(&mut vec_u32_be);
        vec.append(&mut vec_u64_be);

        /*
        let mut decoder = Decoder::new(vec);
        assert_eq!(decoder.drain_i16(), 32767);
        assert_eq!(decoder.drain_i24(), 0x00FFFFFFi32);
        assert_eq!(decoder.drain_i32(), 0x1234abcdi32);
        assert_eq!(decoder.drain_i64(), 0x1234abcd1234abcdi64);

        assert_eq!(decoder.drain_f32(), std::f32::consts::PI);
        assert_eq!(decoder.drain_f64(), std::f64::consts::PI);
        assert_eq!(decoder.drain_u16(), 65535);
        assert_eq!(decoder.drain_u24(), 0x00ffffff);
        assert_eq!(decoder.drain_u32(), 4294967295);
        assert_eq!(decoder.drain_u64(), 18446744073709551615u64);*/

        let core = core::Core::new();
        let mut buf = std::fs::read("D:/test.flv").unwrap();
        // dbg!(buf[0..64].to_vec());
        let mut buf: VecDeque<u8> = buf.into_iter().collect();
        core.borrow_mut().push_data(&mut buf);
        core.borrow_mut().start_decoding();
    }
}
