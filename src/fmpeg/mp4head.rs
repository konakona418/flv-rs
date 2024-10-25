pub struct Utils;
impl Utils {
    #[inline]
    pub fn str_to_char_array(s: &String) -> [char; 4] {
        let mut result = ['\0', '\0', '\0', '\0'];
        for (i, c) in s.chars().enumerate() {
            result[i] = c;
        }
        result
    }

    #[inline]
    pub fn slice_to_char_array(slice: &str) -> [char; 4] {
        let mut result = ['\0', '\0', '\0', '\0'];
        for (i, c) in slice.chars().enumerate() {
            result[i] = c;
        }
        result
    }
}

pub struct U24 {
    pub value: u32,
}

impl U24 {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    #[inline]
    pub fn from(value: u32) -> U24 {
        Self::new(value & 0x00FFFFFF)
    }

    #[inline]
    pub fn to_u32(&self) -> u32 {
        self.value & 0x00FFFFFF
    }
}

impl Default for U24 {
    #[inline]
    fn default() -> Self {
        Self::new(0)
    }
}

impl ISerializable for U24 {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let mut result = vec![];
        result.extend_from_slice(&self.to_u32().to_be_bytes()[1..]);
        assert_eq!(result.len(), 3);
        result
    }

    #[inline]
    fn size(&self) -> u32 {
        3
    }
}

pub struct FixedPoint16 {
    pub integer: u8,
    pub fraction: u8,
}

impl FixedPoint16 {
    #[inline]
    fn new(integer: u8, fraction: u8) -> Self {
        Self { integer, fraction }
    }

    #[inline]
    fn from(float: f32) -> FixedPoint16 {
        Self::new(float.trunc() as u8, ((float - float.trunc()) * 256.0) as u8)
    }

    #[inline]
    fn to_float(&self) -> f32 {
        self.integer as f32 + self.fraction as f32 / 256.0
    }
}

impl Default for FixedPoint16 {
    #[inline]
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl ISerializable for FixedPoint16 {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let mut result = vec![];
        result.push(self.integer);
        result.push(self.fraction);
        assert_eq!(result.len(), 2);
        result
    }

    #[inline]
    fn size(&self) -> u32 {
        2
    }
}

pub struct FixedPoint32 {
    pub integer: u16,
    pub fraction: u16,
}

impl FixedPoint32 {
    #[inline]
    fn new(integer: u16, fraction: u16) -> Self {
        Self { integer, fraction }
    }

    #[inline]
    fn from(float: f32) -> FixedPoint32 {
        Self::new(float.trunc() as u16, ((float - float.trunc()) * 65536.0) as u16)
    }

    #[inline]
    fn to_float(&self) -> f32 {
        self.integer as f32 + self.fraction as f32 / 65536.0
   }
}

impl Default for FixedPoint32 {
    #[inline]
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl ISerializable for FixedPoint32 {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let mut result = vec![];
        result.extend_from_slice(&self.integer.to_be_bytes());
        result.extend_from_slice(&self.fraction.to_be_bytes());
        assert_eq!(result.len(), 4);
        result
    }

    #[inline]
    fn size(&self) -> u32 {
        4
    }
}

pub struct Box {
    pub size: u32,
    pub box_type: [char; 4],
}

pub struct FullBox {
    pub size: u32,
    pub box_type: [char; 4],
    pub version: u8,
    pub flags: U24,
}

pub trait ISerializable {
    fn serialize(&mut self) -> Vec<u8>;
    fn size(&self) -> u32;
}

pub struct FileTypeBox {
    pub size: u32,
    pub box_type: [char; 4],

    pub major_brand: [char; 4],
    pub minor_version: u32,
    pub compatible_brands: Vec<[char; 4]>,
}

impl Default for FileTypeBox {
    fn default() -> Self {
        Self {
            size: 0,
            box_type: ['f', 't', 'y', 'p'],
            major_brand: ['m', 'p', '4', '2'],
            minor_version: 0,
            compatible_brands: vec![['m', 'p', '4', '2']],
        }
    }
}

impl ISerializable for FileTypeBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.major_brand.map(|c| c as u8));
        result.extend_from_slice(&self.minor_version.to_be_bytes());
        for brand in &self.compatible_brands {
            result.extend_from_slice(&brand.map(|c| c as u8));
        }
        result
    }

    #[inline]
    fn size(&self) -> u32 {
        16 + 4 * self.compatible_brands.len() as u32
    }
}

pub struct FileTypeBoxBuilder {
    pub major_brand: [char; 4],
    pub minor_version: u32,
    pub compatible_brands: Vec<[char; 4]>,
}

impl FileTypeBoxBuilder {
    pub fn new() -> Self {
        Self {
            major_brand: ['m', 'p', '4', '2'],
            minor_version: 0,
            compatible_brands: vec![['m', 'p', '4', '2']],
        }
    }

    #[inline]
    pub fn major_brand(mut self, major_brand: &String) {
        self.major_brand = Utils::slice_to_char_array(major_brand)
    }

    #[inline]
    pub fn minor_version(mut self, minor_version: u32) {
        self.minor_version = minor_version;
    }

    #[inline]
    pub fn compatible_brand(mut self, compatible_brand: &String) {
        self.compatible_brands.push(Utils::slice_to_char_array(compatible_brand));
    }

    #[inline]
    pub fn compatible_brands(mut self, compatible_brands: Vec<&String>) {
        for brand in compatible_brands {
            self.compatible_brands.push(Utils::slice_to_char_array(brand));
        }
    }

    pub fn build(self) -> FileTypeBox {
        let mut box_instance = FileTypeBox {
            size: 0,
            box_type: ['f', 't', 'y', 'p'],
            major_brand: self.major_brand,
            minor_version: self.minor_version,
            compatible_brands: self.compatible_brands,
        };
        box_instance.size = box_instance.size();
        assert_ne!(box_instance.size, 0);
        // should not have zero size!
        box_instance
    }
}

pub enum MovieHeaderBox {
    V0(MovieHeaderBoxV0),
    V1(MovieHeaderBoxV1),
}

impl ISerializable for MovieHeaderBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        match self {
            MovieHeaderBox::V0(box_instance) => box_instance.serialize(),
            MovieHeaderBox::V1(box_instance) => box_instance.serialize(),
        }
    }

    #[inline]
    fn size(&self) -> u32 {
        match self {
            MovieHeaderBox::V0(box_instance) => box_instance.size(),
            MovieHeaderBox::V1(box_instance) => box_instance.size(),
        }
    }
}

pub struct MovieHeaderBoxV0 {
    pub size: u32,
    pub box_type: [char; 4],
    pub version: u8,
    pub flags: U24,

    pub creation_time: u32,
    pub modification_time: u32,
    pub timescale: u32,
    pub duration: u32,

    pub rate: FixedPoint32,
    pub volume: FixedPoint16,
    pub reserved: [u8; 10],

    pub matrix: [u8; 36],

    pub preview_time: u32,
    pub preview_duration: u32,
    pub poster_time: u32,
    pub selection_time: u32,
    pub selection_duration: u32,
    pub current_time: u32,
    pub next_track_id: u32
}

const MATRIX: [u8; 36] = [
    0x00, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x40, 0x00, 0x00, 0x00
];

impl MovieHeaderBoxV0 {
    pub fn new(creation_time: u32, modification_time: u32, timescale: u32, duration: u32, rate: FixedPoint32, volume: FixedPoint16, next_track_id: u32) -> Self {
        Self {
            size: 0,
            box_type: ['m', 'v', 'h', 'd'],
            version: 0,
            flags: U24::from(0),
            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
            volume,
            reserved: [0; 10],
            matrix: MATRIX,
            preview_time: 0,
            preview_duration: 0,
            poster_time: 0,
            selection_time: 0,
            selection_duration: 0,
            current_time: 0,
            // next_track_id = 1 + the count of all tracks (video + audio)
            next_track_id
        }
    }
}

impl ISerializable for MovieHeaderBoxV0 {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));

        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.flags.serialize());

        result.extend_from_slice(&self.creation_time.to_be_bytes());
        result.extend_from_slice(&self.modification_time.to_be_bytes());
        result.extend_from_slice(&self.timescale.to_be_bytes());
        result.extend_from_slice(&self.duration.to_be_bytes());
        result.extend_from_slice(&self.rate.serialize());
        result.extend_from_slice(&self.volume.serialize());

        result.extend_from_slice(&self.reserved);

        result.extend_from_slice(&MATRIX);

        result.extend_from_slice(&self.preview_time.to_be_bytes());
        result.extend_from_slice(&self.preview_duration.to_be_bytes());
        result.extend_from_slice(&self.poster_time.to_be_bytes());
        result.extend_from_slice(&self.selection_time.to_be_bytes());
        result.extend_from_slice(&self.selection_duration.to_be_bytes());
        result.extend_from_slice(&self.current_time.to_be_bytes());
        result.extend_from_slice(&self.next_track_id.to_be_bytes());
        assert_eq!(result.len(), 108);
        result
    }

    #[inline]
    fn size(&self) -> u32 {
        108
    }
}

pub struct MovieHeaderBoxV0Builder {
    pub creation_time: u32,
    pub modification_time: u32,
    pub timescale: u32,
    pub duration: u32,
    pub rate: FixedPoint32,
    pub volume: FixedPoint16,
    pub next_track_id: u32
}

impl MovieHeaderBoxV0Builder {
    pub fn new() -> Self {
        Self {
            creation_time: 0,
            modification_time: 0,
            timescale: 0,
            duration: 0,
            rate: FixedPoint32::new(1, 0),
            volume: FixedPoint16::new(1, 0),
            next_track_id: 0
        }
    }

    #[inline]
    pub fn creation_time(mut self, creation_time: u32) -> Self {
        self.creation_time = creation_time;
        self
    }

    #[inline]
    pub fn modification_time(mut self, modification_time: u32) -> Self {
        self.modification_time = modification_time;
        self
    }

    #[inline]
    pub fn timescale(mut self, timescale: u32) -> Self {
        // todo: timescale conversion (from milli sec of FLV to timescale of fMpeg4)
        self.timescale = timescale;
        self
    }

    #[inline]
    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }

    #[inline]
    pub fn rate(mut self, rate: f32) -> Self {
        self.rate = FixedPoint32::from(rate);
        self
    }

    #[inline]
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = FixedPoint16::from(volume);
        self
    }

    /// Note: this is not the count of all tracks (video + audio),
    /// but the count of all tracks (video + audio) PLUS 1!!!
    /// to be specific, the mpeg4 decoder automatically allocates a unique id for a certain track,
    /// this attribute tells the decoder that there's already 'count' tracks.
    /// and the newly added track should possess the id 'next_track_id'.
    /// however in flv.js, this value is set to 0xFF 0xFF 0xFF 0xFF.
    #[inline]
    pub fn next_track_id(mut self, next_track_id: u32) -> Self {
        self.next_track_id = next_track_id;
        self
    }

    pub fn build(self) -> MovieHeaderBoxV0 {
        MovieHeaderBoxV0::new(
            self.creation_time,
            self.modification_time,
            self.timescale,
            self.duration,
            self.rate,
            self.volume,
            self.next_track_id
        )
    }
}

pub struct MovieHeaderBoxV1 {
    pub size: u32,
    pub box_type: [char; 4],

    pub version: u8,
    pub flags: U24,

    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,

    pub rate: FixedPoint32,
    pub volume: FixedPoint16,
    pub reserved: [u8; 10],

    pub matrix: [u8; 36],

    pub preview_time: u32,
    pub preview_duration: u32,
    pub poster_time: u32,
    pub selection_time: u32,
    pub selection_duration: u32,
    pub current_time: u32,
    pub next_track_id: u32
}

impl MovieHeaderBoxV1 {
    pub fn new(creation_time: u64, modification_time: u64, timescale: u32, duration: u64, rate: FixedPoint32, volume: FixedPoint16, next_track_id: u32) -> Self {
        Self {
            size: 0,
            box_type: ['m', 'v', 'h', 'd'],
            version: 1,
            flags: U24::from(0),

            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
            volume,
            reserved: [0; 10],
            matrix: MATRIX,
            preview_time: 0,
            preview_duration: 0,
            poster_time: 0,
            selection_time: 0,
            selection_duration: 0,
            current_time: 0,
            next_track_id
        }
    }
}

impl ISerializable for MovieHeaderBoxV1 {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.flags.serialize());

        result.extend_from_slice(&self.creation_time.to_be_bytes());
        result.extend_from_slice(&self.modification_time.to_be_bytes());
        result.extend_from_slice(&self.timescale.to_be_bytes());
        result.extend_from_slice(&self.duration.to_be_bytes());
        result.extend_from_slice(&self.rate.serialize());
        result.extend_from_slice(&self.volume.serialize());
        result.extend_from_slice(&self.reserved);
        result.extend_from_slice(&self.matrix);
        result.extend_from_slice(&self.preview_time.to_be_bytes());
        result.extend_from_slice(&self.preview_duration.to_be_bytes());
        result.extend_from_slice(&self.poster_time.to_be_bytes());
        result.extend_from_slice(&self.selection_time.to_be_bytes());
        result.extend_from_slice(&self.selection_duration.to_be_bytes());
        result.extend_from_slice(&self.current_time.to_be_bytes());
        result.extend_from_slice(&self.next_track_id.to_be_bytes());
        assert_eq!(result.len(), 120);
        result
    }

    fn size(&self) -> u32 {
        120
    }
}

pub struct MovieHeaderBoxV1Builder {
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,
    pub rate: FixedPoint32,
    pub volume: FixedPoint16,
    pub next_track_id: u32
}

impl MovieHeaderBoxV1Builder {
    pub fn new() -> Self {
        Self {
            creation_time: 0,
            modification_time: 0,
            timescale: 0,
            duration: 0,
            rate: FixedPoint32::new(1, 0),
            volume: FixedPoint16::new(1, 0),
            next_track_id: 0
        }
    }

    #[inline]
    pub fn creation_time(mut self, creation_time: u64) -> Self {
        self.creation_time = creation_time;
        self
    }

    #[inline]
    pub fn modification_time(mut self, modification_time: u64) -> Self {
        self.modification_time = modification_time;
        self
    }

    #[inline]
    pub fn timescale(mut self, timescale: u32) -> Self {
        // todo: timescale conversion (from milli sec of FLV to timescale of fMpeg4)
        self.timescale = timescale;
        self
    }

    #[inline]
    pub fn duration(mut self, duration: u64) -> Self {
        self.duration = duration;
        self
    }

    #[inline]
    pub fn rate(mut self, rate: f32) -> Self {
        self.rate = FixedPoint32::from(rate);
        self
    }

    #[inline]
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = FixedPoint16::from(volume);
        self
    }

    #[inline]
    pub fn next_track_id(mut self, next_track_id: u32) -> Self {
        self.next_track_id = next_track_id;
        self
    }

    pub fn build(self) -> MovieHeaderBoxV1 {
        MovieHeaderBoxV1::new(
            self.creation_time,
            self.modification_time,
            self.timescale,
            self.duration,
            self.rate,
            self.volume,
            self.next_track_id
        )
    }
}

pub struct TrackBox {
    pub size: u32,
    pub box_type: [char; 4],

    pub track_header_box: TrackHeaderBox,
    pub media_box: MediaBox,
}

impl TrackBox {
    pub fn new(track_header_box: TrackHeaderBox, media_box: MediaBox) -> Self {
        Self {
            size: 0,
            box_type: ['t', 'r', 'a', 'k'],
            track_header_box,
            media_box
        }
    }
}

pub enum TrackHeaderBox {
    V0(TrackHeaderBoxV0),
    V1(TrackHeaderBoxV1),
}

impl ISerializable for TrackHeaderBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        match self {
            TrackHeaderBox::V0(box_) => box_.serialize(),
            TrackHeaderBox::V1(box_) => box_.serialize(),
        }
    }

    fn size(&self) -> u32 {
        match self {
            TrackHeaderBox::V0(box_) => box_.size(),
            TrackHeaderBox::V1(box_) => box_.size(),
        }
    }
}

pub struct TrackHeaderBoxV0 {
    pub size: u32,
    pub box_type: [char; 4],
    pub version: u8,
    pub flags: U24,

    pub creation_time: u32,
    pub modification_time: u32,
    pub track_id: u32,

    pub reserved: [u8; 4],

    pub duration: u32,

    pub reserved2: [u8; 8],

    pub layer: i16,
    pub alternate_group: i16,
    pub volume: i16,

    pub reserved3: [u8; 2],

    pub matrix: [u8; 36],

    pub width: FixedPoint32,
    pub height: FixedPoint32,
}

impl TrackHeaderBoxV0 {
    pub fn new(creation_time: u32, modification_time: u32, track_id: u32, duration: u32, width: FixedPoint32, height: FixedPoint32) -> Self {
        Self {
            size: 0,
            box_type: ['t', 'k', 'h', 'd'],
            version: 0,
            flags: U24::from(7), // todo: what does flags stand for?

            creation_time,
            modification_time,
            track_id,
            reserved: [0; 4],
            duration,
            reserved2: [0; 8],
            layer: 0,
            alternate_group: 0,
            volume: 0,
            reserved3: [0; 2],
            matrix: [0; 36],
            width,
            height,
        }
    }
}

impl ISerializable for TrackHeaderBoxV0 {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.flags.serialize());

        result.extend_from_slice(&self.creation_time.to_be_bytes());
        result.extend_from_slice(&self.modification_time.to_be_bytes());
        result.extend_from_slice(&self.track_id.to_be_bytes());
        result.extend_from_slice(&self.reserved);
        result.extend_from_slice(&self.duration.to_be_bytes());
        result.extend_from_slice(&self.reserved2);
        result.extend_from_slice(&self.layer.to_be_bytes());
        result.extend_from_slice(&self.alternate_group.to_be_bytes());
        result.extend_from_slice(&self.volume.to_be_bytes());
        result.extend_from_slice(&self.reserved3);
        result.extend_from_slice(&MATRIX);
        result.extend_from_slice(&self.width.serialize());
        result.extend_from_slice(&self.height.serialize());
        assert_eq!(result.len(), 92);
        result
    }

    fn size(&self) -> u32 {
        92
    }
}

pub struct TrackHeaderBoxV0Builder {
    pub creation_time: u32,
    pub modification_time: u32,
    pub track_id: u32,
    pub duration: u32,
    pub width: FixedPoint32,
    pub height: FixedPoint32
}

impl TrackHeaderBoxV0Builder {
    pub fn new() -> Self {
        Self {
            creation_time: 0,
            modification_time: 0,
            track_id: 0,
            duration: 0,
            width: FixedPoint32::new(1, 0),
            height: FixedPoint32::new(1, 0)
        }
    }

    #[inline]
    pub fn creation_time(mut self, creation_time: u32) -> Self {
        self.creation_time = creation_time;
        self
    }

    #[inline]
    pub fn modification_time(mut self, modification_time: u32) -> Self {
        self.modification_time = modification_time;
        self
    }

    #[inline]
    pub fn track_id(mut self, track_id: u32) -> Self {
        self.track_id = track_id;
        self
    }

    #[inline]
    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }

    #[inline]
    pub fn width(mut self, width: FixedPoint32) -> Self {
        self.width = width;
        self
    }

    #[inline]
    pub fn height(mut self, height: FixedPoint32) -> Self {
        self.height = height;
        self
    }

    pub fn build(self) -> TrackHeaderBoxV0 {
        TrackHeaderBoxV0::new(
            self.creation_time,
            self.modification_time,
            self.track_id,
            self.duration,
            self.width,
            self.height
        )
    }
}

pub struct TrackHeaderBoxV1 {
    pub size: u32,
    pub box_type: [char; 4],
    pub version: u8,
    pub flags: U24,

    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,

    pub reserved: [u8; 4],

    pub duration: u64,

    pub reserved2: [u8; 8],

    pub layer: i16,
    pub alternate_group: i16,
    pub volume: i16,

    pub reserved3: [u8; 2],

    pub matrix: [u8; 36],

    pub width: FixedPoint32,
    pub height: FixedPoint32,
}

impl TrackHeaderBoxV1 {
    pub fn new(creation_time: u64, modification_time: u64, track_id: u32, duration: u64, width: FixedPoint32, height: FixedPoint32) -> Self {
        Self {
            size: 0,
            box_type: ['m', 'd', 'h', 'd'],
            version: 1,
            flags: U24::from(7), // todo: what does flags stand for?

            creation_time,
            modification_time,
            track_id,
            reserved: [0; 4],
            duration,
            reserved2: [0; 8],
            layer: 0,
            alternate_group: 0,
            volume: 0,
            reserved3: [0; 2],
            matrix: [0; 36],
            width,
            height,
        }
    }
}

impl ISerializable for TrackHeaderBoxV1 {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.flags.serialize());

        result.extend_from_slice(&self.creation_time.to_be_bytes());
        result.extend_from_slice(&self.modification_time.to_be_bytes());
        result.extend_from_slice(&self.track_id.to_be_bytes());
        result.extend_from_slice(&self.reserved);
        result.extend_from_slice(&self.duration.to_be_bytes());
        result.extend_from_slice(&self.reserved2);
        result.extend_from_slice(&self.layer.to_be_bytes());
        result.extend_from_slice(&self.alternate_group.to_be_bytes());
        result.extend_from_slice(&self.volume.to_be_bytes());
        result.extend_from_slice(&self.reserved3);
        result.extend_from_slice(&MATRIX);
        result.extend_from_slice(&self.width.serialize());
        result.extend_from_slice(&self.height.serialize());
        assert_eq!(result.len(), 104);
        result
    }

    fn size(&self) -> u32 {
        104
    }
}

pub struct MediaBox {
    pub size: u32,
    pub box_type: [char; 4],

    pub media_header: MediaHeaderBoxV0,
    pub xmedia_handler_box: XMediaHandlerBox,
    pub media_info_box: MediaInfoBox
}

impl MediaBox {
    pub fn new(media_header: MediaHeaderBoxV0, xmedia_handler_box: XMediaHandlerBox, media_info_box: MediaInfoBox) -> Self {
        Self {
            size: 0,
            box_type: ['m', 'd', 'i', 'a'],
            media_header,
            xmedia_handler_box,
            media_info_box
        }
    }
}

impl ISerializable for MediaBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.media_header.serialize());
        result.extend_from_slice(&self.xmedia_handler_box.serialize());
        result.extend_from_slice(&self.media_info_box.serialize());
        assert_eq!(result.len(), self.size() as usize);
        result
    }

    #[inline]
    fn size(&self) -> u32 {
        8 + self.media_header.size() + self.xmedia_handler_box.size() + self.media_info_box.size()
    }
}

pub struct MediaHeaderBoxV0 {
    pub size: u32,
    pub box_type: [char; 4],
    pub version: u8,
    pub flags: U24,

    pub creation_time: u32,
    pub modification_time: u32,
    pub timescale: u32,
    pub duration: u32,

    pub language: u16,
    pub quality: u16,
}

impl MediaHeaderBoxV0 {
    pub fn new(creation_time: u32, modification_time: u32, timescale: u32, duration: u32, language: u16, quality: u16) -> Self {
        Self {
            size: 0,
            box_type: ['m', 'h', 'd', 'r'],
            version: 0,
            flags: U24::from(0),

            creation_time,
            modification_time,
            timescale,
            duration,
            language,
            quality,
        }
    }
}

impl ISerializable for MediaHeaderBoxV0 {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.flags.serialize());

        result.extend_from_slice(&self.creation_time.to_be_bytes());
        result.extend_from_slice(&self.modification_time.to_be_bytes());
        result.extend_from_slice(&self.timescale.to_be_bytes());
        result.extend_from_slice(&self.duration.to_be_bytes());
        result.extend_from_slice(&self.language.to_be_bytes());
        result.extend_from_slice(&self.quality.to_be_bytes());
        assert_eq!(result.len(), 32);
        result
    }

    fn size(&self) -> u32 {
        32
    }
}

pub struct MediaHeaderBoxV0Builder {
    creation_time: u32,
    modification_time: u32,
    timescale: u32,
    duration: u32,
    language: u16,
    quality: u16,
}

impl MediaHeaderBoxV0Builder {
    #[inline]
    pub fn new() -> Self {
        Self {
            creation_time: 0,
            modification_time: 0,
            timescale: 0,
            duration: 0,
            language: 0x55C4u16, // undetermined.
            quality: 0,
        }
    }

    #[inline]
    pub fn creation_time(mut self, creation_time: u32) -> Self {
        self.creation_time = creation_time;
        self
    }

    #[inline]
    pub fn modification_time(mut self, modification_time: u32) -> Self {
        self.modification_time = modification_time;
        self
    }

    #[inline]
    pub fn timescale(mut self, timescale: u32) -> Self {
        self.timescale = timescale;
        self
    }

    #[inline]
    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }

    #[inline]
    pub fn language(mut self, language: u16) -> Self {
        self.language = language;
        self
    }

    #[inline]
    pub fn quality(mut self, quality: u16) -> Self {
        self.quality = quality;
        self
    }

    #[inline]
    pub fn build(self) -> MediaHeaderBoxV0 {
        MediaHeaderBoxV0::new(self.creation_time, self.modification_time, self.timescale, self.duration, self.language, self.quality)
    }
}

pub struct HandlerBox {
    pub size: u32,
    pub box_type: [char; 4],
    pub version: u8,
    pub flags: U24,

    pub component_type: [u8; 4],
    pub handler_type: [char; 4],
    pub reserved: [u8; 12],
    pub name: [u8; 13]
}

impl HandlerBox {
    pub fn new(handler_type: [char; 4], name: String) -> Self {
        Self {
            size: 0,
            box_type: ['h', 'd', 'l', 'r'],
            version: 0,
            flags: U24::from(0),

            component_type: [0; 4],
            handler_type,
            reserved: [0; 12],
            name: name.as_bytes().try_into().unwrap()
        }
    }
}

impl ISerializable for HandlerBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.flags.serialize());

        result.extend_from_slice(&self.component_type);
        result.extend_from_slice(&self.handler_type.map(|c| c as u8));
        result.extend_from_slice(&self.reserved);
        result.extend_from_slice(&self.name);
        assert_eq!(result.len(), 45);
        result
    }

    fn size(&self) -> u32 {
        45
    }
}

pub enum HandlerType {
    Video,
    Audio,
}

impl HandlerType {
    pub fn into(self) -> Vec<u8> {
        match self {
            HandlerType::Video => {
                HandlerBox::new(['v', 'i', 'd', 'e'], "VideoHandler\x00".to_string()).serialize()
            }
            HandlerType::Audio => {
                HandlerBox::new(['s', 'o', 'u', 'n'], "SoundHandler\x00".to_string()).serialize()
            }
        }
    }
}

pub struct MediaInfoBox {
    pub size: u32,
    pub box_type: [char; 4],

    xmedia_handler_box: XMediaHandlerBox,
    data_information_box: DataInformationBox,
    sample_table_box: SampleBoxTableBox,
}

impl MediaInfoBox {
    pub fn new(xmedia_handler_box: XMediaHandlerBox, data_information_box: DataInformationBox, sample_table_box: SampleBoxTableBox) -> Self {
        Self {
            size: 0,
            box_type: ['m', 'i', 'n', 'f'],

            xmedia_handler_box,
            data_information_box,
            sample_table_box,
        }
    }
}

impl ISerializable for MediaInfoBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));

        result.extend_from_slice(&self.xmedia_handler_box.serialize());
        result.extend_from_slice(&self.data_information_box.serialize());
        result.extend_from_slice(&self.sample_table_box.serialize());
        assert_eq!(result.len(), self.size as usize);
        result
    }

    fn size(&self) -> u32 {
        self.xmedia_handler_box.size() + self.data_information_box.size() + self.sample_table_box.size() + 8
    }
}

pub enum XMediaHandlerBox {
    Video(VideoMediaHandlerBox),
    Audio(AudioMediaHandlerBox),
}

impl ISerializable for XMediaHandlerBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        match self {
            XMediaHandlerBox::Video(video) => video.serialize(),
            XMediaHandlerBox::Audio(audio) => audio.serialize(),
        }
    }

    fn size(&self) -> u32 {
        match self {
            XMediaHandlerBox::Video(video) => video.size(),
            XMediaHandlerBox::Audio(audio) => audio.size(),
        }
    }
}

pub struct VideoMediaHandlerBox;

impl VideoMediaHandlerBox {
    pub fn new() -> Self {
        Self
    }
}

impl ISerializable for VideoMediaHandlerBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let data: [u8; 20] = [
            0x00, 0x00, 0x00, 0x14, // size = 20
            0x76, 0x6D, 0x68, 0x64, // "vmhd"
            0x00, 0x00, 0x00, 0x01, // version =0, flags = 1
            0x00, 0x00, 0x00, 0x00, // graphics mode = 0
            0x00, 0x00, 0x00, 0x00, // opcolor = 0
        ];
        data.to_vec()
    }

    fn size(&self) -> u32 {
        20
    }
}

pub struct AudioMediaHandlerBox;

impl ISerializable for AudioMediaHandlerBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let data: [u8; 0x10] = [
            0x00, 0x00, 0x00, 0x10, // size = 16
            0x73, 0x6D, 0x68, 0x64, // "smhd"
            0x00, 0x00, 0x00, 0x00, // version = 0, flags = 0
            0x00, 0x00, 0x00, 0x00, // balance = 0
        ];
        data.to_vec()
    }

    fn size(&self) -> u32 {
        16
    }
 }

pub struct DataInformationBox;
impl ISerializable for DataInformationBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let data: [u8; 28] = [
            0x00, 0x00, 0x00, 0x1C, // size = 28
            0x64, 0x72, 0x65, 0x66, // "dinf"
            0x00, 0x00, 0x00, 0x00, // version = 0, flags = 0
            0x00, 0x00, 0x00, 0x01, // entry count = 1
            0x00, 0x00, 0x00, 0x0C, // box size = 12
            0x75, 0x72, 0x6C, 0x20, // "url "
            0x00, 0x00, 0x00, 0x01, // flags
        ];
        data.to_vec()
    }

    fn size(&self) -> u32 {
        28
    }
}

pub struct SampleBoxTableBox {
    pub size: u32,
    pub box_type: [char; 4],

    sample_description_table_box: SampleDescriptionTableBox,
    time_to_sample_box: TimeToSampleBox,
    sample_to_chunk_box: SampleToChunkBox,
    sample_size_box: SampleSizeBox,
    chunk_offset_box: ChunkOffsetBox,
}

impl ISerializable for SampleBoxTableBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));

        result.extend_from_slice(&self.sample_description_table_box.serialize());
        result.extend_from_slice(&self.time_to_sample_box.serialize());
        result.extend_from_slice(&self.sample_to_chunk_box.serialize());
        result.extend_from_slice(&self.sample_size_box.serialize());
        result.extend_from_slice(&self.chunk_offset_box.serialize());
        assert_eq!(result.len(), self.size as usize);
        result
    }

    fn size(&self) -> u32 {
        self.sample_description_table_box.size() +
            self.time_to_sample_box.size() +
            self.sample_to_chunk_box.size() +
            self.sample_size_box.size() +
            self.chunk_offset_box.size() +
            8
    }
}

pub struct SampleDescriptionTableBox {
    pub size: u32,
    pub box_type: [char; 4],
    pub version: u8,
    pub flags: U24,
    pub entry_count: u32,

    pub sample_description_table: Vec<SubSampleDescriptionTableBox>,
}

impl ISerializable for SampleDescriptionTableBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.flags.serialize());
        result.extend_from_slice(&self.entry_count.to_be_bytes());

        for entry in &mut self.sample_description_table {
            result.extend_from_slice(&entry.serialize());
        }
        assert_eq!(result.len(), self.size as usize);
        result
    }

    #[inline]
    fn size(&self) -> u32 {
        let mut size = 16;
        for entry in &self.sample_description_table {
            size += entry.size();
        }
        size
    }
}

pub struct SampleDescriptionTableBoxBuilder {
    size: u32,
    box_type: [char; 4],
    version: u8,
    flags: U24,
    entry_count: u32,
    sample_description_table: Vec<SubSampleDescriptionTableBox>,
}

impl SampleDescriptionTableBoxBuilder {
    pub fn new() -> Self {
        Self {
            size: 0,
            box_type: ['s', 't', 's', 'd'],
            version: 0,
            flags: U24::new(0),
            entry_count: 0,
            sample_description_table: vec![]
        }
    }

    pub fn add_sample_description_table_box(mut self, sample_description_table_box: SubSampleDescriptionTableBox) -> Self {
        self.sample_description_table.push(sample_description_table_box);
        self.entry_count += 1;
        self
    }

    pub fn build(self) -> SampleDescriptionTableBox {
        SampleDescriptionTableBox {
            size: self.size,
            box_type: self.box_type,
            version: self.version,
            flags: self.flags,
            entry_count: self.entry_count,
            sample_description_table: self.sample_description_table
        }
    }
}

pub enum SubSampleDescriptionTableBox {
    Mp4a(Mp4aDescriptionBox),
    Mp3(Mp3DescriptionBox),
    Avc1(Avc1DescriptionBox),
}

impl ISerializable for SubSampleDescriptionTableBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        match self {
            SubSampleDescriptionTableBox::Mp4a(mp4a) => mp4a.serialize(),
            SubSampleDescriptionTableBox::Mp3(mp3) => mp3.serialize(),
            SubSampleDescriptionTableBox::Avc1(avc1) => avc1.serialize(),
        }
    }

    fn size(&self) -> u32 {
        match self {
            SubSampleDescriptionTableBox::Mp4a(mp4a) => mp4a.size(),
            SubSampleDescriptionTableBox::Mp3(mp3) => mp3.size(),
            SubSampleDescriptionTableBox::Avc1(avc1) => avc1.size(),
        }
    }
}

pub mod aac_utils {
    use crate::fmpeg::mp4head::ISerializable;
    use crate::io;
    use crate::io::bit::UIntParserEndian;

    pub enum AacObjectType {
        Null,
        AacMain,
        AacLc,
        AacSsr,
        AacLtp,
        AacSbr,
        AacScalable
    }

    impl Into<u16> for AacObjectType {
        #[inline]
        fn into(self) -> u16 {
            match self {
                AacObjectType::Null => 0,
                AacObjectType::AacMain => 1,
                AacObjectType::AacLc => 2, // Low complexity
                AacObjectType::AacSsr => 3, // Scalable sample rate
                AacObjectType::AacLtp => 4, // Long term prediction
                AacObjectType::AacSbr => 5, // HE AAC
                AacObjectType::AacScalable => 6
            }
        }
    }

    impl AacObjectType {
        #[inline]
        pub fn as_u16(&self) -> u16 {
            match self {
                AacObjectType::Null => 0,
                AacObjectType::AacMain => 1,
                AacObjectType::AacLc => 2, // Low complexity
                AacObjectType::AacSsr => 3, // Scalable sample rate
                AacObjectType::AacLtp => 4, // Long term prediction
                AacObjectType::AacSbr => 5, // HE AAC
                AacObjectType::AacScalable => 6
            }
        }
    }

    impl From<u16> for AacObjectType {
        /// Note: this is for the decoder, not the encoder!!
        /// do not use it directly for flv metadata!
        #[inline]
        fn from(value: u16) -> Self {
            match value {
                0 => AacObjectType::Null,
                1 => AacObjectType::AacMain,
                2 => AacObjectType::AacLc,
                3 => AacObjectType::AacSsr,
                4 => AacObjectType::AacLtp,
                5 => AacObjectType::AacSbr,
                6 => AacObjectType::AacScalable,
               _ => panic!("Invalid AAC object type")
            }
        }
    }

    pub enum SamplingFreqIndex {
        Freq96000,
        Freq88200,
        Freq64000,
        Freq48000,
        Freq44100,
        Freq32000,
        Freq24000,
        Freq22050,
        Freq16000,
        Freq12000,
        Freq11025,
        Freq8000,
        Freq7350,
        FreqExplicit,
    }

    impl Into<u16> for SamplingFreqIndex {
        #[inline]
        fn into(self) -> u16 {
            match self {
                SamplingFreqIndex::Freq96000 => 0x0,
                SamplingFreqIndex::Freq88200 => 0x1,
                SamplingFreqIndex::Freq64000 => 0x2,
                SamplingFreqIndex::Freq48000 => 0x3,
                SamplingFreqIndex::Freq44100 => 0x4,
                SamplingFreqIndex::Freq32000 => 0x5,
                SamplingFreqIndex::Freq24000 => 0x6,
                SamplingFreqIndex::Freq22050 => 0x7,
                SamplingFreqIndex::Freq16000 => 0x8,
                SamplingFreqIndex::Freq12000 => 0x9,
                SamplingFreqIndex::Freq11025 => 0xa,
                SamplingFreqIndex::Freq8000  => 0xb,
                SamplingFreqIndex::Freq7350  => 0xc,
                SamplingFreqIndex::FreqExplicit => 0xf,
            }
        }
    }

    impl SamplingFreqIndex {
        #[inline]
        pub fn as_u16(&self) -> u16 {
            match self {
                SamplingFreqIndex::Freq96000 => 0x0,
                SamplingFreqIndex::Freq88200 => 0x1,
                SamplingFreqIndex::Freq64000 => 0x2,
                SamplingFreqIndex::Freq48000 => 0x3,
                SamplingFreqIndex::Freq44100 => 0x4,
                SamplingFreqIndex::Freq32000 => 0x5,
                SamplingFreqIndex::Freq24000 => 0x6,
                SamplingFreqIndex::Freq22050 => 0x7,
                SamplingFreqIndex::Freq16000 => 0x8,
                SamplingFreqIndex::Freq12000 => 0x9,
                SamplingFreqIndex::Freq11025 => 0xa,
                SamplingFreqIndex::Freq8000  => 0xb,
                SamplingFreqIndex::Freq7350  => 0xc,
                SamplingFreqIndex::FreqExplicit => 0xf,
            }
        }
    }

    pub enum ChannelConfig {
        AacExtension,
        Mono,
        Stereo,
        Three,
        Four,
        Five,
        Six,
        Seven,
        Eight,
    }

    impl Into<u16> for ChannelConfig {
        #[inline]
        fn into(self) -> u16 {
            match self {
                ChannelConfig::AacExtension => 0x0,
                ChannelConfig::Mono => 0x1,
                ChannelConfig::Stereo => 0x2,
                ChannelConfig::Three => 0x3,
                ChannelConfig::Four => 0x4,
                ChannelConfig::Five => 0x5,
                ChannelConfig::Six => 0x6,
                ChannelConfig::Seven => 0x7,
                ChannelConfig::Eight => 0x8,
            }
        }
    }

    impl ChannelConfig {
        #[inline]
        pub fn as_u16(&self) -> u16 {
            match self {
                ChannelConfig::AacExtension => 0x0,
                ChannelConfig::Mono => 0x1,
                ChannelConfig::Stereo => 0x2,
                ChannelConfig::Three => 0x3,
                ChannelConfig::Four => 0x4,
                ChannelConfig::Five => 0x5,
                ChannelConfig::Six=> 0x6,
                ChannelConfig::Seven => 0x7,
                ChannelConfig::Eight => 0x8,
            }
        }
    }

    pub enum FrameLengthFlag {
        Sample1024_0,
        Sample960_1,
    }

    impl Into<u16> for FrameLengthFlag {
        #[inline]
        fn into(self) -> u16 {
            match self {
                FrameLengthFlag::Sample1024_0 => 0x0,
                FrameLengthFlag::Sample960_1 => 0x1,
            }
        }
    }

    impl FrameLengthFlag {
        #[inline]
        pub fn as_u16(&self) -> u16 {
            match self {
                FrameLengthFlag::Sample1024_0 => 0x0,
                FrameLengthFlag::Sample960_1 => 0x1,
            }
        }
    }

    pub enum CoreCoderDependentFlag {
        No,
        Yes,
    }

    impl Into<u16> for CoreCoderDependentFlag {
        #[inline]
        fn into(self) -> u16 {
            match self {
                CoreCoderDependentFlag::No => 0x0,
                CoreCoderDependentFlag::Yes => 0x1,
            }
        }
    }

    impl CoreCoderDependentFlag {
        #[inline]
        pub fn as_u16(&self) -> u16 {
            match self {
                CoreCoderDependentFlag::No => 0x0,
                CoreCoderDependentFlag::Yes => 0x1,
            }
        }
    }

    pub enum ExtensionFlag {
        No,
        Yes,
    }

    impl Into<u16> for ExtensionFlag {
        #[inline]
        fn into(self) -> u16 {
            match self {
                ExtensionFlag::No => 0x0,
                ExtensionFlag::Yes => 0x1,
            }
        }
    }

    impl ExtensionFlag {
        #[inline]
        pub fn as_u16(&self) -> u16 {
            match self {
                ExtensionFlag::No => 0x0,
                ExtensionFlag::Yes => 0x1,
            }
        }
    }

    pub const GA_SPEC_CONF: [u8; 3] = [0x06, 0x01, 0x02];
    // see also ISO/IEC 14496-3:2009

    pub enum AacAudioSpecConfLike {
        AacAudioSpecificConfig(AacAudioSpecificConfigBox),
        AacAudioSpecificConfigLike(Vec<u8>),
    }

    impl ISerializable for AacAudioSpecConfLike {
        #[inline]
        fn serialize(&mut self) -> Vec<u8> {
            match self {
                AacAudioSpecConfLike::AacAudioSpecificConfig(box_) => box_.serialize(),
                AacAudioSpecConfLike::AacAudioSpecificConfigLike(data) => data.clone(),
            }
        }

        #[inline]
        fn size(&self) -> u32 {
            match self {
                AacAudioSpecConfLike::AacAudioSpecificConfig(box_) => box_.size(),
                AacAudioSpecConfLike::AacAudioSpecificConfigLike(data) => data.len() as u32,
            }
        }
    }

    pub struct AacAudioSpecificConfigBox {
        pub aac_object_type: AacObjectType,
        pub sampling_freq_index: SamplingFreqIndex,
        pub channel_config: ChannelConfig,
        pub frame_length_flag: FrameLengthFlag,
        pub core_coder_dependent_flag: CoreCoderDependentFlag,
        pub extension_flag: ExtensionFlag,
    }

    impl ISerializable for AacAudioSpecificConfigBox {
        #[inline]
        fn serialize(&mut self) -> Vec<u8> {
            let mut result = io::bit::U16BitIO::new(0, UIntParserEndian::BigEndian);
            result.write_range(0, 4, self.aac_object_type.as_u16());
            result.write_range(5, 8, self.sampling_freq_index.as_u16());
            result.write_range(9, 12, self.channel_config.as_u16());
            result.write_at(13, self.frame_length_flag.as_u16() != 0);
            result.write_at(14, self.core_coder_dependent_flag.as_u16() != 0);
            result.write_at(15, self.extension_flag.as_u16() != 0);

            result.data.to_vec()
        }

        #[inline]
        fn size(&self) -> u32 {
            2
        }
    }

    pub struct AacAudioSpecificConfigBoxBuilder {
        aac_object_type: AacObjectType,
        sampling_freq_index: SamplingFreqIndex,
        channel_config: ChannelConfig,
        frame_length_flag: FrameLengthFlag,
        core_coder_dependent_flag: CoreCoderDependentFlag,
        extension_flag: ExtensionFlag,
    }

    impl AacAudioSpecificConfigBoxBuilder {
        #[inline]
        pub fn new() -> AacAudioSpecificConfigBoxBuilder {
            AacAudioSpecificConfigBoxBuilder {
                aac_object_type: AacObjectType::AacLc,
                sampling_freq_index: SamplingFreqIndex::Freq48000,
                channel_config: ChannelConfig::Stereo,
                frame_length_flag: FrameLengthFlag::Sample1024_0,
                core_coder_dependent_flag: CoreCoderDependentFlag::No,
                extension_flag: ExtensionFlag::No
            }
        }

        #[inline]
        pub fn set_aac_object_type(mut self, aac_object_type: AacObjectType) -> Self {
            self.aac_object_type = aac_object_type;
            self
        }

        #[inline]
        pub fn set_sampling_freq_index(mut self, sampling_freq_index: SamplingFreqIndex) -> Self {
            self.sampling_freq_index = sampling_freq_index;
            self
        }

        #[inline]
        pub fn set_channel_config(mut self, channel_config: ChannelConfig) -> Self {
            self.channel_config = channel_config;
            self
        }

        #[inline]
        pub fn set_frame_length_flag(mut self, frame_length_flag: FrameLengthFlag) -> Self {
            self.frame_length_flag = frame_length_flag;
            self
        }

        #[inline]
        pub fn set_core_coder_dependent_flag(mut self, core_coder_dependent_flag: CoreCoderDependentFlag) -> Self {
            self.core_coder_dependent_flag = core_coder_dependent_flag;
            self
        }

        #[inline]
        pub fn set_extension_flag(mut self, extension_flag: ExtensionFlag) -> Self {
            self.extension_flag = extension_flag;
            self
        }

        #[inline]
        pub fn build(mut self) -> AacAudioSpecificConfigBox {
            AacAudioSpecificConfigBox {
                aac_object_type: self.aac_object_type,
                sampling_freq_index: self.sampling_freq_index,
                channel_config: self.channel_config,
                frame_length_flag: self.frame_length_flag,
                core_coder_dependent_flag: self.core_coder_dependent_flag,
                extension_flag: self.extension_flag
            }
        }
    }
}

pub struct AudioExtendedDescriptionBox {
    pub size: u32,
    pub box_type: [char; 4],
    pub version: u8,
    pub flags: U24,

    pub aac_audio_specific_config: aac_utils::AacAudioSpecConfLike,
}

impl AudioExtendedDescriptionBox {
    #[inline]
    pub fn new(spec_config: aac_utils::AacAudioSpecConfLike) -> AudioExtendedDescriptionBox {
        AudioExtendedDescriptionBox {
            size: 0,
            box_type: ['e', 's', 'd', 's'],
            version: 0,
            flags: U24::from(0),
            aac_audio_specific_config: spec_config
        }
    }
}

impl Default for AudioExtendedDescriptionBox {
    #[inline]
    fn default() -> AudioExtendedDescriptionBox {
        AudioExtendedDescriptionBox {
            size: 0,
            box_type: ['e', 's', 'd', 's'],
            version: 0,
            flags: U24::from(0),
            aac_audio_specific_config: aac_utils::AacAudioSpecConfLike::AacAudioSpecificConfig(
                aac_utils::AacAudioSpecificConfigBox {
                    aac_object_type: aac_utils::AacObjectType::AacLc,
                    sampling_freq_index: aac_utils::SamplingFreqIndex::Freq44100,
                    channel_config: aac_utils::ChannelConfig::Stereo,
                    frame_length_flag: aac_utils::FrameLengthFlag::Sample1024_0,
                    core_coder_dependent_flag: aac_utils::CoreCoderDependentFlag::No,
                    extension_flag: aac_utils::ExtensionFlag::No,
            })
        }
    }
}

impl ISerializable for AudioExtendedDescriptionBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.flags.serialize());

        result.push(0x03); // descriptor
        result.push(0x17 + self.aac_audio_specific_config.size() as u8); // size

        result.push(0x00);
        result.push(0x01);
        // es id

        result.push(0x00); // stream priority

        result.push(0x04); // descriptor type
        result.push(0x0F + self.aac_audio_specific_config.size() as u8); // size
        result.push(0x40); // codec: mpeg4a
        result.push(0x15); // stream type: Audio
        result.extend_from_slice(&[0u8; 11]);

        result.push(0x05); // descriptor type
        result.push(self.aac_audio_specific_config.size() as u8);
        result.extend_from_slice(&self.aac_audio_specific_config.serialize());

        result.extend_from_slice(&aac_utils::GA_SPEC_CONF);
        assert_eq!(result.len(), 37 + self.aac_audio_specific_config.size() as usize);
        result
    }

    #[inline]
    fn size(&self) -> u32 {
        37 + self.aac_audio_specific_config.size()
    }
}

pub struct Mp4aDescriptionBox {
    pub size: u32,
    pub box_type: [char; 4],
    pub reserved: [u8; 6],
    pub data_reference_index: u16,
    pub version: u16,
    pub revision_level: u16,
    pub max_packet_size: u32,
    pub num_audio_channels: u16,
    pub sample_size: u16,
    pub compression_id: u16,
    pub packet_size: u16,
    pub sample_rate: FixedPoint32,

    pub aac_extended_description: AudioExtendedDescriptionBox,
}

impl ISerializable for Mp4aDescriptionBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));

        // ------------ can be optimized by setting to 0 --------------
        result.extend_from_slice(&self.reserved);
        result.extend_from_slice(&self.data_reference_index.to_be_bytes());
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.revision_level.to_be_bytes());
        result.extend_from_slice(&self.max_packet_size.to_be_bytes());
        // ------------------------------------------------------------

        result.extend_from_slice(&self.num_audio_channels.to_be_bytes());
        result.extend_from_slice(&self.sample_size.to_be_bytes());

        // ------------ can be optimized by setting to 0 --------------
        result.extend_from_slice(&self.compression_id.to_be_bytes());
        result.extend_from_slice(&self.packet_size.to_be_bytes());
        // ------------------------------------------------------------

        result.extend_from_slice(&self.sample_rate.serialize());

        result.extend_from_slice(&self.aac_extended_description.serialize());
        result
    }

    fn size(&self) -> u32 {
        36 + self.aac_extended_description.size()
    }
}

impl Mp4aDescriptionBox {
    pub fn new(sample_rate: f32, num_audio_channels: u16, spec_config: aac_utils::AacAudioSpecConfLike) -> Self {
        Self {
            size: 0,
            box_type: ['m', 'p', '4', 'a'],
            reserved: [0; 6],
            data_reference_index: 1,
            version: 0,
            revision_level: 0,
            max_packet_size: 0,
            num_audio_channels,
            sample_size: 16,
            compression_id: 0,
            packet_size: 0,
            sample_rate: FixedPoint32::from(sample_rate),
            aac_extended_description: AudioExtendedDescriptionBox::new(spec_config),
        }
    }
}

pub struct Mp3DescriptionBox {
    pub size: u32,
    pub box_type: [char; 4],
    pub reserved: [u8; 6],
    pub data_reference_index: u16,
    pub version: u16,
    pub revision_level: u16,
    pub max_packet_size: u32,
    pub num_audio_channels: u16,
    pub sample_size: u16,
    pub compression_id: u16,
    pub packet_size: u16,
    pub sample_rate: FixedPoint32,
}

impl ISerializable for Mp3DescriptionBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));

        // ------------ can be optimized by setting to 0 --------------
        result.extend_from_slice(&self.reserved);
        result.extend_from_slice(&self.data_reference_index.to_be_bytes());
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.revision_level.to_be_bytes());
        result.extend_from_slice(&self.max_packet_size.to_be_bytes());
        // ------------------------------------------------------------

        result.extend_from_slice(&self.num_audio_channels.to_be_bytes());
        result.extend_from_slice(&self.sample_size.to_be_bytes());

        // ------------ can be optimized by setting to 0 --------------
        result.extend_from_slice(&self.compression_id.to_be_bytes());
        result.extend_from_slice(&self.packet_size.to_be_bytes());
        // ------------------------------------------------------------

        result.extend_from_slice(&self.sample_rate.serialize());
        assert_eq!(result.len(), 36);

        result
    }

    fn size(&self) -> u32 {
        36
    }
}

pub mod avc1_utils {
    use crate::fmpeg::mp4head::ISerializable;

    pub enum AvcCBoxLike {
        AvcCBoxLike(Vec<u8>)
    }

    impl ISerializable for AvcCBoxLike {
        #[inline]
        fn serialize(&mut self) -> Vec<u8> {
            match self {
                Self::AvcCBoxLike(data) => data.clone(),
            }
        }

        fn size(&self) -> u32 {
            match self {
                Self::AvcCBoxLike(data) => data.len() as u32,
            }
        }
    }
}

pub struct Avc1DescriptionBox {
    pub size: u32,
    pub box_type: [char; 4],

    pub reserved: [u8; 6],

    pub data_reference_index: u16,
    pub version: u16,
    pub revision_level: u16,
    pub max_packet_size: u32,

    pub temporal_quality: u32,
    pub spatial_quality: u32,
    pub width: u16,
    pub height: u16,
    pub horiz_resolution: FixedPoint32,
    pub vert_resolution: FixedPoint32,
    pub data_size: u32,
    pub frame_count: u16,
    pub compressor_name: [u8; 32],
    pub depth: u16,
    pub color_table_id: i16,

    pub avcc_box: avc1_utils::AvcCBoxLike,
}

impl Avc1DescriptionBox {
    pub fn new(width: u16, height: u16, avcc_box: avc1_utils::AvcCBoxLike) -> Self {
        Self {
            size: 0,
            box_type: ['a', 'v', 'c', '1'],
            reserved: [0; 6],
            data_reference_index: 1,
            version: 0,
            revision_level: 0,
            max_packet_size: 0,
            temporal_quality: 0,
            spatial_quality: 0,
            width,
            height,
            horiz_resolution: FixedPoint32::from(72.0),
            vert_resolution: FixedPoint32::from(72.0),
            data_size: 0,
            frame_count: 1,
            compressor_name: [0; 32],
            depth: 24,
            color_table_id: -1,
            avcc_box
        }
    }
}

impl ISerializable for Avc1DescriptionBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        self.size = self.size();

        let mut result = vec![];
        result.extend_from_slice(&self.size.to_be_bytes());
        result.extend_from_slice(&self.box_type.map(|c| c as u8));
        result.extend_from_slice(&self.reserved);
        result.extend_from_slice(&self.data_reference_index.to_be_bytes());
        result.extend_from_slice(&self.version.to_be_bytes());
        result.extend_from_slice(&self.revision_level.to_be_bytes());
        result.extend_from_slice(&self.max_packet_size.to_be_bytes());

        result.extend_from_slice(&self.temporal_quality.to_be_bytes());
        result.extend_from_slice(&self.spatial_quality.to_be_bytes());
        result.extend_from_slice(&self.width.to_be_bytes());
        result.extend_from_slice(&self.height.to_be_bytes());
        result.extend_from_slice(&self.horiz_resolution.serialize());
        result.extend_from_slice(&self.vert_resolution.serialize());
        result.extend_from_slice(&self.data_size.to_be_bytes());
        result.extend_from_slice(&self.frame_count.to_be_bytes());
        result.extend_from_slice(&self.compressor_name);
        result.extend_from_slice(&self.depth.to_be_bytes());
        result.extend_from_slice(&self.color_table_id.to_be_bytes());

        result.extend_from_slice(&self.avcc_box.serialize());

        assert_eq!(result.len(), 86 + self.avcc_box.size() as usize);
        result
    }

    fn size(&self) -> u32 {
        86 + self.avcc_box.size()
    }
}

pub struct Avc1DescriptionBoxBuilder {
    width: u16,
    height: u16,
    avcc_box: avc1_utils::AvcCBoxLike,
}

impl Avc1DescriptionBoxBuilder {
    pub fn new(width: u16, height: u16, avcc_box: avc1_utils::AvcCBoxLike) -> Self {
        Self {
            width,
            height,
            avcc_box,
        }
    }

    pub fn build(self) -> Avc1DescriptionBox {
        Avc1DescriptionBox::new(self.width, self.height, self.avcc_box)
    }
}

pub struct TimeToSampleBox;

impl TimeToSampleBox {
    pub fn new(sample_count: u32) -> Self {
        Self {}
    }
}

impl ISerializable for TimeToSampleBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let data: [u8; 16] = [
            0x00, 0x00, 0x00, 0x10, // size = 16
            0x73, 0x74, 0x74, 0x73, // "stts"
            0x00, 0x00, 0x00, 0x00, // version= 0, flags = 0
            0x00, 0x00, 0x00, 0x00, // entry count = 1
        ];
        data.to_vec()
    }

    fn size(&self) -> u32 {
        16
    }
}

pub struct SampleToChunkBox;

impl SampleToChunkBox {
    pub fn new(sample_count: u32) -> Self {
        Self {}
    }
}

impl ISerializable for SampleToChunkBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let data: [u8; 16] = [
            0x00, 0x00, 0x00, 0x10, // size = 16
            0x73, 0x74, 0x73, 0x63, // "stsc"
            0x00, 0x00, 0x00, 0x00, // version=0, flags = 0
            0x00, 0x00, 0x00, 0x00,
        ];
        data.to_vec()
    }

    fn size(&self) -> u32 {
        16
    }
}

pub struct SampleSizeBox;

impl SampleSizeBox {
    pub fn new(sample_size: u32) -> Self {
        Self {}
    }
}

impl ISerializable for SampleSizeBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let data: [u8; 20] = [
            0x00, 0x00, 0x00, 0x14, // size = 20
            0x73, 0x74, 0x73, 0x7A, // "stsz"
            0x00, 0x00, 0x00, 0x00, // version=0, flags = 0
            0x00, 0x00, 0x00, 0x00, // sample size = 0
            0x00, 0x00, 0x00, 0x00, // sample count = 0
        ];
        data.to_vec()
    }

    fn size(&self) -> u32 {
        20
    }
}

pub struct ChunkOffsetBox;

impl ChunkOffsetBox {
    pub fn new(chunk_offset: u32) -> Self {
        Self {}
    }
}

impl ISerializable for ChunkOffsetBox {
    #[inline]
    fn serialize(&mut self) -> Vec<u8> {
        let data: [u8; 16] = [
            0x00, 0x00, 0x00, 0x10, // size = 16
            0x73, 0x74, 0x63, 0x6F, // "stco"
            0x00, 0x00, 0x00, 0x00, // version=0, flags = 0
            0x00, 0x00, 0x00, 0x00,
        ];
        data.to_vec()
    }

    fn size(&self) -> u32 {
        16
    }
}

// above are the implementation of file head.