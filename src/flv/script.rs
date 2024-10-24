use crate::flv::decoder::Decoder;
use std::io::Error;

pub fn parse_object(data: &mut Decoder) -> Result<ScriptData, Box<dyn std::error::Error>> {
    let data_type = data.drain_u8();
    let value = match data_type {
        0 => ScriptData::Number(data.drain_f64()),
        1 => ScriptData::Boolean(data.drain_u8()),
        2 => ScriptData::String(ScriptDataString::parse_no_marker(data)?),
        3 => ScriptData::Object(ScriptDataObject::parse_no_marker(data)?),

        7 => ScriptData::Reference(data.drain_u16()),
        8 => ScriptData::EcmaArray(ScriptDataEcmaArray::parse_no_marker(data)?),
        9 => ScriptData::ObjectEndMarker,
        10 => ScriptData::StrictArray(ScriptStrictArray::parse_no_marker(data)?),
        11 => ScriptData::Date(ScriptDataDate::parse_no_marker(data)?),
        12 => ScriptData::LongString(ScriptDataLongString::parse_no_marker(data)?),
        _ => {
            println!("Reserved type {}.", data_type);
            ScriptData::NotImplemented
        }
    };
    Ok(value)
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ScriptDataObject {
    pub properties: Vec<ScriptDataObjectProp>,
}

impl ScriptDataObject {

    pub fn parse(data: &mut Decoder) -> Result<ScriptDataObject, Box<dyn std::error::Error>> {
        let type_marker = data.drain_u8();
        if type_marker != 3 {
            return Err(
                Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unable to parse object: Expected type marker Object(3), found something else.",
                ).into()
            );
        }

        ScriptDataObject::parse_no_marker(data)
    }

    pub fn parse_no_marker(data: &mut Decoder) -> Result<ScriptDataObject, Box<dyn std::error::Error>> {
        let mut properties = Vec::new();
        loop {
            let key = ScriptDataString::parse_no_marker(data)?;
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

#[derive(Debug, Clone)]
pub struct ScriptDataObjectProp {
    pub name: ScriptDataString,
    pub value: ScriptData,
}

#[derive(Debug, Clone)]
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
                    format!("Unable to parse string: Expected type marker String(2), found {}.", type_marker),
                ).into()
            );
        }
        let length = data.drain_u16();
        let data = data.drain_bytes_vec(length as usize).into_iter().collect::<Vec<_>>();
        let data = String::from_utf8(data)?;
        Ok(ScriptDataString { length, data })
    }

    pub fn parse_no_marker(data: &mut Decoder) -> Result<ScriptDataString, Box<dyn std::error::Error>> {
        let length = data.drain_u16();
        let data = data.drain_bytes_vec(length as usize).into_iter().collect::<Vec<_>>();
        let data = String::from_utf8(data)?;
        Ok(ScriptDataString { length, data })
    }
}

#[derive(Debug, Clone)]
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
                    format!("Unable to parse long string: Expected type marker LongString(12), found {}.", type_marker),
                ).into()
            );
        }
        Self::parse_no_marker(data)
    }

    pub fn parse_no_marker(data: &mut Decoder) -> Result<ScriptDataLongString, Box<dyn std::error::Error>> {
        let length = data.drain_u32();
        let data = data.drain_bytes_vec(length as usize).into_iter().collect::<Vec<_>>();
        let data = String::from_utf8(data)?;
        Ok(ScriptDataLongString { length, data })
    }
}

#[derive(Debug, Clone)]
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
                    format!("Unable to parse ecma array: Expected type marker EcmaArray(8), found {}.", type_marker),
                ).into()
            );
        }

        Self::parse_no_marker(data)
        // todo: is the last elem of the ecma array the 'end marker'?
        // it seems that the answer is no. but i'm not sure.
    }

    pub fn parse_no_marker(data: &mut Decoder) -> Result<ScriptDataEcmaArray, Box<dyn std::error::Error>> {
        let length = data.drain_u32();
        let mut properties = Vec::with_capacity(length as usize);
        for _ in 0..length + 1 {
            let key = ScriptDataString::parse_no_marker(data)?;
            let data = parse_object(data)?;
            properties.push(ScriptDataObjectProp { name: key, value: data });
        }
        Ok(ScriptDataEcmaArray { length, properties })
    }
}

#[derive(Debug, Clone)]
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
                    format!("Unable to parse strict array: Expected type marker StrictArray(10), found {}.", type_marker),
                ).into()
            );
        }

        Self::parse_no_marker(data)
    }

    pub fn parse_no_marker(data: &mut Decoder) -> Result<ScriptStrictArray, Box<dyn std::error::Error>> {
        let length = data.drain_u32();
        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length + 1 {
            let value = parse_object(data)?;
            values.push(value);
        }
        Ok(ScriptStrictArray { length, values })
    }
}

#[derive(Debug, Clone)]
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
                    format!("Unable to parse date: Expected type marker Date(11), found {}.", type_marker),
                ).into()
            );
        }

        Self::parse_no_marker(data)
    }

    pub fn parse_no_marker(data: &mut Decoder) -> Result<ScriptDataDate, Box<dyn std::error::Error>> {
        let date = data.drain_f64();
        let local_time_offset = data.drain_i16();
        Ok(ScriptDataDate { date, local_time_offset })
    }
}