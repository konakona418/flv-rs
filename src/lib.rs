mod flv;
mod io;
mod core;
mod exchange;
mod fmpeg;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, VecDeque};
    use std::io::Write;
    use crate::flv::decoder::Decoder;
    use crate::flv::tag::TagType;
    use crate::fmpeg::encoder::Encoder;
    use crate::fmpeg::mp4head::{ISerializable, U24};
    use crate::fmpeg::remux_context::{AudioCodecType, RemuxContext, VideoCodecType};
    use crate::io::bit::UIntParserEndian;
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
        let mut decoder = Decoder::new(VecDeque::from(buf));
        dbg!(decoder.decode_header().unwrap());
        for _ in 0..1 {
            decoder.drain_u32();
            dbg!(decoder.decode_tag().unwrap());
        } /**/


        // Note: by the way, till this commit, the decoder (especially the AAC part)
        // works quite well in single thread mode and in unit tests.

        // the Hash and Eq impls for Destination are not tested.
        let map: HashMap<exchange::Destination, String> = HashMap::from([
            (exchange::Destination::Core, "core".to_string()),
            (exchange::Destination::Decoder, "decoder".to_string()),
            (exchange::Destination::Demuxer, "demuxer".to_string()),
        ]);
        assert_eq!(map.get(&exchange::Destination::Core).unwrap(), "core");
        assert_eq!(map.get(&exchange::Destination::Decoder).unwrap(), "decoder");
        assert_eq!(map.get(&exchange::Destination::Demuxer).unwrap(), "demuxer");

        assert_eq!(TagType::Audio, TagType::Audio);
        assert_eq!(TagType::Video, TagType::Video);
        // now it's tested.

        let mut u16io = io::bit::U16BitIO::new(0x1234, UIntParserEndian::BigEndian);
        assert_eq!(u16io.read_at(0), false);
        assert_eq!(u16io.read_at(3), true);
        assert_eq!(u16io.read_at(7), false);
        assert_eq!(u16io.read_at(10), true);
        assert_eq!(u16io.read_at(15), false);

        assert_eq!(u16io.read_range(0, 3), 1);
        assert_eq!(u16io.read_range(4, 7), 2);
        assert_eq!(u16io.read_range(6, 11), 0x23);
        assert_eq!(u16io.read_range(7, 11), 0x03);
        assert_eq!(u16io.read_range(7, 15), 0x34);

        u16io.write_at(0, true);
        u16io.write_at(3, false);
        assert_eq!(u16io.read_range(0, 3), 0b1000);

        u16io.write_range(4, 7, 0b1010);
        assert_eq!(u16io.read_range(4, 7), 0b1010);

        u16io.write_range(10, 15, 0b101010);
        assert_eq!(u16io.read_range(10, 15), 0b101010);

        let mut u16io = io::bit::U16BitIO::new(0b1111000000000000, UIntParserEndian::BigEndian);
        u16io.write_range(0, 4, 0b11011);
        dbg!(u16io.read_range(0, 3));
        dbg!(u16io.read_range(4, 7));
        assert_eq!(u16io.read_range(0, 4), 0b11011);

        // 1101 1000 0000
        // 0000 0010 1001
        // 1101 1010 1001
        u16io.write_range(6, 11, 0b101001);
        dbg!(u16io.read_range(0, 3), 0b1101);
        dbg!(u16io.read_range(4, 7), 0b1010);
        dbg!(u16io.read_range(8, 11), 0b1001);
        assert_eq!(u16io.read_range(12, 15), 0);

        // it seems that u16io module works well.
        let mut u24io = U24::from(0x11123456u32);
        dbg!(u24io.serialize());

        let mut remux_context = RemuxContext::new();
        remux_context.set_header_sent(true);
        remux_context._set_configured(true);
        remux_context.width = 1280f64;
        remux_context.height = 720f64;
        remux_context.fps = 30f64;
        remux_context.fps_num = 30000;
        remux_context.duration_ms = 1000;
        remux_context.has_audio = true;
        remux_context.has_video = true;
        remux_context.audio_codec_id = 2;
        remux_context.audio_codec_type = AudioCodecType::Aac;
        remux_context.audio_sample_rate = 48000;
        remux_context.audio_data_rate = 128;
        remux_context.audio_channels = 2;
        remux_context.video_codec_id = 7;
        remux_context.video_codec_type = VideoCodecType::Avc1;
        remux_context.major_brand = "isom".to_string();
        remux_context.minor_version = 512.to_string();
        remux_context.compatible_brands = vec!["isom".to_string(), "iso6".to_string(), "avc1".to_string(), "mp41".to_string()];

        Encoder::encode_ftyp(&remux_context);
        Encoder::encode_moov(&remux_context);

        let mut vec = vec![];
        vec.append(&mut Encoder::encode_ftyp(&remux_context).serialize());
        vec.append(&mut Encoder::encode_moov(&remux_context).serialize());

        let mut write_file = std::fs::File::create("D:/out.mp4").unwrap();
        write_file.write(&vec).unwrap();
    }
}
