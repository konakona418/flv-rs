use std::fs::File;
use std::io::Read;

pub trait IReader {
    fn read_bit(&mut self) -> bool;
    fn read_u8(&mut self) -> u8;
    fn read_u16(&mut self) -> u16;
    fn read_u32(&mut self) -> u32;
    fn read_u64(&mut self) -> u64;
    fn read_float32(&mut self) -> f32;
    fn read_float64(&mut self) -> f64;
    fn read_bytes(&mut self, size: usize) -> Vec<u8>;
    fn read_string(&mut self) -> String;
}

pub struct LocalReader {
    data: Vec<u8>,
    file: File
}

impl LocalReader {
    pub fn new(filename: &str) -> LocalReader {
        LocalReader {
            data: Vec::new(),
            file: File::open(filename).unwrap()
        }
    }

    pub fn read_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.file.read_to_end(&mut self.data)?;
        Ok(())
    }

    pub fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), std::io::Error> {
        self.file.read_exact(buffer)?;
        Ok(())
    }
}