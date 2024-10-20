use std::io::Error;
use crate::flv::decoder::Decoder;

pub fn parse_object(data: &mut Decoder) -> Result<ScriptData, Box<dyn std::error::Error>> {
    let data_type = data.drain_u8();
    let value = match data_type {
        0 => ScriptData::Number(data.drain_f64()),
        1 => ScriptData::Boolean(data.drain_u8()),
        2 => ScriptData::String(ScriptDataString::parse(data)?),
        3 => ScriptData::Object(ScriptDataObject::parse(data)?),

        7 => ScriptData::Reference(data.drain_u16()),
        8 => ScriptData::EcmaArray(ScriptDataEcmaArray::parse(data)?),
        9 => ScriptData::ObjectEndMarker,
        10 => ScriptData::StrictArray(ScriptStrictArray::parse(data)?),
        11 => ScriptData::Date(ScriptDataDate::parse(data)?),
        12 => ScriptData::LongString(ScriptDataLongString::parse(data)?),
        _ => {
            println!("Reserved type {}.", data_type);
            ScriptData::NotImplemented
        },
    };
    Ok(value)
}

pub struct ScriptTagBody {
    pub name: ScriptDataString,
    pub value: ScriptDataEcmaArray,
}

impl ScriptTagBody {
    pub fn parse(data: &mut Decoder) -> Result<ScriptTagBody, Box<dyn std::error::Error>> {
        let name = ScriptDataString::parse(data)?;
        let value = ScriptDataEcmaArray::parse(data)?;
        Ok(ScriptTagBody { name, value })
    }
}

pub enum ScriptData {
    Number(f64),
    Boolean(u8),
    String(ScriptDataString),
    Object(ScriptDataObject),
    MovieClip,
    Null,
    Undefined,
    Reference(u16),
    EcmaArray(ScriptDataEcmaArray),
    ObjectEndMarker,
    StrictArray(ScriptStrictArray),
    Date(ScriptDataDate),
    LongString(ScriptDataLongString),
    NotImplemented,
}

pub struct ScriptDataObject {
    pub properties: Vec<ScriptDataObjectProp>,
}

impl ScriptDataObject {
    pub fn parse(data: &mut Decoder) -> Result<ScriptDataObject, Box<dyn std::error::Error>> {
        let mut properties = Vec::new();
        loop {
            let key = ScriptDataString::parse(data)?;
            let data = parse_object(data)?;
            if let ScriptData::ObjectEndMarker = data {
                properties.push(ScriptDataObjectProp { name: key, value: data });
                break;
            } else {
                properties.push(ScriptDataObjectProp { name: key, value: data });
            }
        }
        Ok(ScriptDataObject { properties })
    }
}

pub struct ScriptDataObjectProp {
    pub name: ScriptDataString,
    pub value: ScriptData,
}

pub struct ScriptDataString {
    pub length: u16,
    pub data: String,
}

impl ScriptDataString {
    pub fn parse(data: &mut Decoder) -> Result<ScriptDataString, Box<dyn std::error::Error>> {
        let type_marker = data.drain_u8();
        if type_marker != 2 {
            return Err(
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unable to parse string: Expected type marker String(2), found {}.", type_marker)
                ).into()
            );
        }
        let length = data.drain_u16();
        let data = data.drain_bytes_vec(length as usize).into_iter().collect::<Vec<_>>();
        let data = String::from_utf8(data)?;
        Ok(ScriptDataString { length, data })
    }
}

pub struct ScriptDataLongString {
    pub length: u32,
    pub data: String,
}

impl ScriptDataLongString {
    pub fn parse(data: &mut Decoder) -> Result<ScriptDataLongString, Box<dyn std::error::Error>> {
        let type_marker = data.drain_u8();
        if type_marker != 12 {
            return Err(
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unable to parse long string: Expected type marker LongString(12), found {}.", type_marker)
                ).into()
           );
        }
        let length = data.drain_u32();
        let data = data.drain_bytes_vec(length as usize).into_iter().collect::<Vec<_>>();
        let data = String::from_utf8(data)?;
        Ok(ScriptDataLongString { length, data })
    }
}

pub struct ScriptDataEcmaArray {
    pub length: u32,
    pub properties: Vec<ScriptDataObjectProp>,
}

impl ScriptDataEcmaArray {
    pub fn parse(data: &mut Decoder) -> Result<ScriptDataEcmaArray, Box<dyn std::error::Error>> {
        let type_marker = data.drain_u8();
        if type_marker != 8 {
            return Err(
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unable to parse ecma array: Expected type marker EcmaArray(8), found {}.", type_marker)
                ).into()
            );
        }

        let length = data.drain_u32();
        let mut properties = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let key = ScriptDataString::parse(data)?;
            let data = parse_object(data)?;
            if let ScriptData::ObjectEndMarker = data {
                properties.push(ScriptDataObjectProp { name: key, value: data });
                break;
            } else {
                properties.push(ScriptDataObjectProp { name: key, value: data });
            }
        }
        Ok(ScriptDataEcmaArray { length, properties })
    }
}

pub struct ScriptStrictArray {
    pub length: u32,
    pub values: Vec<ScriptData>,
}

impl ScriptStrictArray {
    pub fn parse(data: &mut Decoder) -> Result<ScriptStrictArray, Box<dyn std::error::Error>> {
        let type_marker = data.drain_u8();
        if type_marker != 10 {
            return Err(
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unable to parse strict array: Expected type marker StrictArray(10), found {}.", type_marker)
                ).into()
            );
        }
        let length = data.drain_u32();
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            let value = parse_object(data)?;
            values.push(value);
        }
        Ok(ScriptStrictArray { length, values })
    }
}

 pub struct ScriptDataDate {
    pub date: f64,
    pub local_time_offset: i16,
}

impl ScriptDataDate {
    pub fn parse(data: &mut Decoder) -> Result<ScriptDataDate, Box<dyn std::error::Error>> {
        let type_marker = data.drain_u8();
        if type_marker != 11 {
            return Err(
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unable to parse date: Expected type marker Date(11), found {}.", type_marker)
                ).into()
            );
        }
        let date = data.drain_f64();
        let local_time_offset = data.drain_i16();
        Ok(ScriptDataDate { date, local_time_offset })
    }
}