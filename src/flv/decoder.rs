use crate::exchange::{Destination, ExchangeRegistrable, Packed, PackedContent, PackedContentToDecoder, PackedContentToDemuxer};
use crate::flv::header::{AudioTagHeader, EncryptionTagHeader, FilterParameters, FlvHeader, TagHeader, VideoTagHeader};
use crate::flv::script::ScriptTagBody;
use crate::flv::tag::{EncryptedTagBody, NormalTagBody, Tag, TagBody, TagType};
use crate::io::bit::BitIO;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

pub struct Decoder {
    data: VecDeque<u8>,
    previous_tag_size: u32,
    channel_exchange: Option<mpsc::Sender<Packed>>,
    channel_receiver: mpsc::Receiver<PackedContent>,
    channel_sender: mpsc::Sender<PackedContent>,
    decoding: bool,
}

impl ExchangeRegistrable for Decoder {
    fn set_exchange(&mut self, sender: mpsc::Sender<Packed>) {
        self.channel_exchange = Some(sender);
    }

    fn get_sender(&self) -> mpsc::Sender<PackedContent> {
        self.channel_sender.clone()
    }

    fn get_self_as_destination(&self) -> Destination {
        Destination::Decoder
    }
}

impl Decoder {
    pub fn new(data: VecDeque<u8>) -> Self {
        let (channel_sender, channel_receiver) = mpsc::channel();
        Decoder {
            data,
            previous_tag_size: 0,
            channel_exchange: None,
            channel_receiver,
            channel_sender,
            decoding: false,
        }
    }

    pub fn push_data(&mut self, data: &mut VecDeque<u8>) {
        self.data.append(data);
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.data.extend(bytes)
    }

    #[inline]
    pub fn drain_u8(&mut self) -> u8 {
        self.data.pop_front().unwrap()
    }

    #[inline]
    pub fn drain_bytes<const SIZE: usize>(&mut self) -> [u8; SIZE] {
        let mut result = [0; SIZE];
        for i in 0..SIZE {
            result[i] = self.drain_u8();
        }
        result
    }

    #[inline]
    pub fn drain_bytes_vec(&mut self, size: usize) -> Vec<u8> {
        let drained = self.data.drain(0..size).collect::<Vec<_>>();
        drained
    }

    #[inline]
    pub fn drain_bytes_deque(&mut self, size: usize) -> VecDeque<u8> {
        let drained = self.data.drain(0..size).collect::<VecDeque<_>>();
        drained
    }

    #[inline]
    pub fn drain_u16_le(&mut self) -> u16 {
        let mut result = 0;
        result |= self.drain_u8() as u16;
        result |= (self.drain_u8() as u16) << 8;
        result
    }

    #[inline]
    pub fn drain_u16(&mut self) -> u16 {
        let mut result = 0;
        result |= (self.drain_u8() as u16) << 8;
        result |= self.drain_u8() as u16;
        result
    }

    #[inline]
    pub fn drain_u24_le(&mut self) -> u32 {
        let mut result = 0;
        result |= self.drain_u8() as u32;
        result |= (self.drain_u8() as u32) << 8;
        result |= (self.drain_u8() as u32) << 16;
        result
    }

    #[inline]
    pub fn drain_u24(&mut self) -> u32 {
        let mut result = 0;
        result |= (self.drain_u8() as u32) << 16;
        result |= (self.drain_u8() as u32) << 8;
        result |= self.drain_u8() as u32;
        result
    }

    #[inline]
    pub fn drain_u32_le(&mut self) -> u32 {
        let mut result = 0;
        result |= self.drain_u8() as u32;
        result |= (self.drain_u8() as u32) << 8;
        result |= (self.drain_u8() as u32) << 16;
        result |= (self.drain_u8() as u32) << 24;
        result
    }

    #[inline]
    pub fn drain_u32(&mut self) -> u32 {
        let mut result = 0;
        result |= (self.drain_u8() as u32) << 24;
        result |= (self.drain_u8() as u32) << 16;
        result |= (self.drain_u8() as u32) << 8;
        result |= self.drain_u8() as u32;
        result
    }

    #[inline]
    pub fn drain_u64(&mut self) -> u64 {
        let mut result = 0;
        result |= (self.drain_u8() as u64) << 56;
        result |= (self.drain_u8() as u64) << 48;
        result |= (self.drain_u8() as u64) << 40;
        result |= (self.drain_u8() as u64) << 32;
        result |= (self.drain_u8() as u64) << 24;
        result |= (self.drain_u8() as u64) << 16;
        result |= (self.drain_u8() as u64) << 8;
        result |= self.drain_u8() as u64;
        result
    }

    #[inline]
    pub fn drain_i8(&mut self) -> i8 {
        self.drain_u8() as i8
    }

    #[inline]
    pub fn drain_i16(&mut self) -> i16 {
        let mut result = 0;
        result |= (self.drain_u8() as i16) << 8;
        result |= self.drain_u8() as i16;
        result
    }

    #[inline]
    pub fn drain_i24(&mut self) -> i32 {
        let mut result = 0;
        result |= (self.drain_u8() as i32) << 16;
        result |= (self.drain_u8() as i32) << 8;
        result |= self.drain_u8() as i32;
        result
    }

    #[inline]
    pub fn drain_i32(&mut self) -> i32 {
        let mut result = 0;
        result |= (self.drain_u8() as i32) << 24;
        result |= (self.drain_u8() as i32) << 16;
        result |= (self.drain_u8() as i32) << 8;
        result |= self.drain_u8() as i32;
        result
    }

    #[inline]
    pub fn drain_i64(&mut self) -> i64 {
        let mut result = 0;
        result |= (self.drain_u8() as i64) << 56;
        result |= (self.drain_u8() as i64) << 48;
        result |= (self.drain_u8() as i64) << 40;
        result |= (self.drain_u8() as i64) << 32;
        result |= (self.drain_u8() as i64) << 24;
        result |= (self.drain_u8() as i64) << 16;
        result |= (self.drain_u8() as i64) << 8;
        result |= self.drain_u8() as i64;
        result
    }

    #[inline]
    pub fn drain_f64(&mut self) -> f64 {
        let mut result = 0;
        result |= (self.drain_u8() as u64) << 56;
        result |= (self.drain_u8() as u64) << 48;
        result |= (self.drain_u8() as u64) << 40;
        result |= (self.drain_u8() as u64) << 32;
        result |= (self.drain_u8() as u64) << 24;
        result |= (self.drain_u8() as u64) << 16;
        result |= (self.drain_u8() as u64) << 8;
        result |= self.drain_u8() as u64;
        f64::from_bits(result)
    }

    #[inline]
    pub fn drain_f64_le(&mut self) -> f64 {
        let mut result = 0;
        result |= self.drain_u8() as u64;
        result |= (self.drain_u8() as u64) << 8;
        result |= (self.drain_u8() as u64) << 16;
        result |= (self.drain_u8() as u64) << 24;
        result |= (self.drain_u8() as u64) << 32;
        result |= (self.drain_u8() as u64) << 40;
        result |= (self.drain_u8() as u64) << 48;
        result |= (self.drain_u8() as u64) << 56;
        f64::from_bits(result)
    }

    #[inline]
    pub fn drain_f32_le(&mut self) -> f32 {
        let mut result = 0;
        result |= self.drain_u8() as u32;
        result |= (self.drain_u8() as u32) << 8;
        result |= (self.drain_u8() as u32) << 16;
        result |= (self.drain_u8() as u32) << 24;
        f32::from_bits(result)
    }

    #[inline]
    pub fn drain_f32(&mut self) -> f32 {
        let mut result = 0;
        result |= (self.drain_u8() as u32) << 24;
        result |= (self.drain_u8() as u32) << 16;
        result |= (self.drain_u8() as u32) << 8;
        result |= self.drain_u8() as u32;
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

    #[inline]
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
                    NormalTagBody::Audio(self.drain_bytes_deque((data_size as usize) - header_size))
                }
                TagType::Video => {
                    tag_header = TagHeader::Video(VideoTagHeader::parse(self, &mut header_size)?);
                    NormalTagBody::Video(self.drain_bytes_deque((data_size as usize) - header_size))
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

    fn set_decoding(&mut self, flag: bool) {
        self.decoding = flag;
    }

    pub fn decode_body(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Ok(received) = self.channel_receiver.recv() {
                if let PackedContent::ToDecoder(packed_content) = received {
                    match packed_content {
                        PackedContentToDecoder::PushData(mut data) => {
                            self.data.append(&mut data)
                        }
                        PackedContentToDecoder::StartDecoding => {
                            self.set_decoding(true);
                        }
                        PackedContentToDecoder::StopDecoding => {
                            self.set_decoding(false);
                        }
                        PackedContentToDecoder::CloseWorkerThread => {
                            break;
                        }
                        PackedContentToDecoder::Now => {
                            // this will literally do nothing.
                            // just applied to remove potential blockage.
                        }
                    }
                }
            } else {
                // todo: use a better way to replace recv().
                println!("Channel closed.");
                return Ok(());
            }

            'decoding: loop {
                if self.data.is_empty() || (!self.decoding) {
                    break 'decoding;
                }
                self.decode_body_once()?
            }
        }
        Ok(())
    }

    pub fn decode_body_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        const HEADER_SIZE: u32 = 11;

        let previous_tag_size = self.drain_u32();

        //dbg!(previous_tag_size);
        if previous_tag_size == self.previous_tag_size {
            let tag = self.decode_tag()?;
            //dbg!(tag.data_size + HEADER_SIZE);
            self.previous_tag_size = tag.data_size + HEADER_SIZE;

            // dbg!(&tag);
            self.send_tag_to_demuxer(tag)?;
            Ok(())
        } else {
            Err(
                Box::new(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Tag size mismatch: expected {}, read {}.", previous_tag_size, self.previous_tag_size)
                    )
                )
            )
        }
    }

    fn send_to_demuxer(&mut self, pack: Packed) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(e) = self.channel_exchange
            .as_ref()
            .unwrap()
            .send(pack) {
            Err(Box::new(e))
        } else {
            Ok(())
        }
    }

    fn send_tag_to_demuxer(&mut self, tag: Tag) -> Result<(), Box<dyn std::error::Error>> {
        let pack: Packed = Packed {
            packed_routing: Destination::Demuxer,
            packed_content: PackedContent::ToDemuxer(PackedContentToDemuxer::PushTag(tag)),
        };
        self.send_to_demuxer(pack)
    }

    fn send_header_to_demuxer(&mut self, flv_header: FlvHeader) -> Result<(), Box<dyn std::error::Error>> {
        let pack: Packed = Packed {
            packed_routing: Destination::Demuxer,
            packed_content: PackedContent::ToDemuxer(PackedContentToDemuxer::PushFlvHeader(flv_header)),
        };
        self.send_to_demuxer(pack)
    }

    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let flv_header = self.decode_header()?;
        self.send_header_to_demuxer(flv_header)?;
        // todo: use a better way to control the decoding loop.
        self.decode_body()?;
        Ok(())
    }

    /// Launch a worker thread that will read from the stream and send the data to the demuxer.
    /// After calling this method, the decoder instance will be moved away from the main thread.
    /// Instead, use the exchange to manipulate the decoder.
    pub fn launch_worker_thread(mut self) -> JoinHandle<()> {
        thread::spawn(move || {
            self.run().unwrap();
        })
    }
}