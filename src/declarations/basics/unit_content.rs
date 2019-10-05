use std::convert::TryFrom;
use std::fmt::{Display, Error as FormatError, Formatter, Result as FormatResult, Write};
use std::num::ParseIntError;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::declarations::errors::ImmuxResult;
use crate::utils::{
    bool_to_u8, f64_to_u8_array, u8_array_to_f64, u8_to_bool, utf8_to_string, varint_decode,
    varint_encode,
};

const CONTENT_TYPE_VALUE_SEPARATOR: &str = "#";
const CONTENT_NIL_TYPE_STR_PREFIX: &str = "_nil";
const CONTENT_STRING_TYPE_STR_PREFIX: &str = "_str";
const CONTENT_JSON_STRING_TYPE_STR_PREFIX: &str = "_json_str";
const CONTENT_FLOAT64_TYPE_STR_PREFIX: &str = "_f64";
const CONTENT_BOOL_TYPE_STR_PREFIX: &str = "_bool";
const CONTENT_BYTES_TYPE_STR_PREFIX: &str = "_bytes";
const CONTENT_BSON_BYTES_TYPE_STR_PREFIX: &str = "_bson_bytes";

fn get_content_type_str_prefix(unit_content: &UnitContent) -> &str {
    match unit_content {
        UnitContent::Nil => CONTENT_NIL_TYPE_STR_PREFIX,
        UnitContent::String(_string) => CONTENT_STRING_TYPE_STR_PREFIX,
        UnitContent::JsonString(_string) => CONTENT_JSON_STRING_TYPE_STR_PREFIX,
        UnitContent::Float64(_f) => CONTENT_FLOAT64_TYPE_STR_PREFIX,
        UnitContent::Bool(_b) => CONTENT_BOOL_TYPE_STR_PREFIX,
        UnitContent::Bytes(_bytes) => CONTENT_BYTES_TYPE_STR_PREFIX,
        UnitContent::BsonBytes(_bytes) => CONTENT_BSON_BYTES_TYPE_STR_PREFIX,
    }
}

fn encode_hex(bytes: &[u8]) -> Result<String, FormatError> {
    let bytes_str_vec: Result<Vec<String>, FormatError> = bytes
        .iter()
        .map(|byte| {
            let mut s = String::new();
            match write!(&mut s, "{:#04x}", byte) {
                Ok(_) => Ok(s),
                Err(error) => Err(error),
            }
        })
        .collect();

    match bytes_str_vec {
        Ok(vec) => {
            let result: String = vec.join(",");
            return Ok(result);
        }
        Err(error) => {
            return Err(error);
        }
    };
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    //    Input is gonna be a str like "[0x01,0x02,0xff]"
    let bytes_str = s.replace("[", "").replace("]", "");
    let bytes_str_vec: Vec<&str> = bytes_str.split(",").collect();

    bytes_str_vec
        .iter()
        .map(|byte_str| {
            //            byte_str is in the format like 0x03, so we only take the last two chars
            u8::from_str_radix(&byte_str[2..], 16)
        })
        .collect()
}

#[repr(u8)]
pub enum ContentTypeBytePrefix {
    Nil = 0x00,

    String = 0x10,
    Boolean = 0x11,
    Float64 = 0x12,

    JsonString = 0x21,
    BsonBytes = 0x22,

    Bytes = 0xff,
}

impl TryFrom<u8> for ContentTypeBytePrefix {
    type Error = UnitContentError;
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        if byte == ContentTypeBytePrefix::Nil as u8 {
            return Ok(ContentTypeBytePrefix::Nil);
        } else if byte == ContentTypeBytePrefix::Bytes as u8 {
            return Ok(ContentTypeBytePrefix::Bytes);
        } else if byte == ContentTypeBytePrefix::JsonString as u8 {
            return Ok(ContentTypeBytePrefix::JsonString);
        } else if byte == ContentTypeBytePrefix::BsonBytes as u8 {
            return Ok(ContentTypeBytePrefix::BsonBytes);
        } else if byte == ContentTypeBytePrefix::String as u8 {
            return Ok(ContentTypeBytePrefix::String);
        } else if byte == ContentTypeBytePrefix::Boolean as u8 {
            return Ok(ContentTypeBytePrefix::Boolean);
        } else if byte == ContentTypeBytePrefix::Float64 as u8 {
            return Ok(ContentTypeBytePrefix::Float64);
        } else {
            return Err(UnitContentError::UnexpectedTypeBytePrefix(byte));
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UnitContent {
    Nil,

    Bytes(Vec<u8>),
    JsonString(String),
    BsonBytes(Vec<u8>),

    String(String),
    Bool(bool),
    Float64(f64),
}

#[derive(Debug)]
pub enum UnitContentError {
    UnexpectedTypeBytePrefix(u8),
    UnexpectedTypeStringPrefix,
    ParseStringError,
    EmptyInput,
    MissingDataBytes,
    UnexpectedLengthBytes,
}

impl UnitContent {
    pub fn marshal(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(1);
        match &self {
            UnitContent::Nil => result.push(ContentTypeBytePrefix::Nil as u8),
            UnitContent::Bool(boolean) => {
                result.push(ContentTypeBytePrefix::Boolean as u8);
                result.push(bool_to_u8(*boolean));
            }
            UnitContent::Float64(number_f64) => {
                result.push(ContentTypeBytePrefix::Float64 as u8);
                result.extend_from_slice(&f64_to_u8_array(*number_f64));
            }
            UnitContent::Bytes(bytes) => {
                result.push(ContentTypeBytePrefix::Bytes as u8);
                result.extend_from_slice(&varint_encode(bytes.len() as u64));
                result.extend_from_slice(bytes)
            }
            UnitContent::JsonString(string) => {
                let bytes = string.as_bytes();
                result.push(ContentTypeBytePrefix::JsonString as u8);
                result.extend_from_slice(&varint_encode(bytes.len() as u64));
                result.extend_from_slice(bytes)
            }
            UnitContent::BsonBytes(bytes) => {
                result.push(ContentTypeBytePrefix::BsonBytes as u8);
                result.extend_from_slice(&varint_encode(bytes.len() as u64));
                result.extend_from_slice(bytes)
            }
            UnitContent::String(string) => {
                let bytes = string.as_bytes();
                result.push(ContentTypeBytePrefix::String as u8);
                result.extend_from_slice(&varint_encode(bytes.len() as u64));
                result.extend_from_slice(bytes)
            }
        }
        return result;
    }
    pub fn parse(data: &[u8]) -> ImmuxResult<(Self, usize)> {
        match data.get(0) {
            None => return Err(UnitContentError::EmptyInput.into()),
            Some(first_byte) => {
                let type_prefix = ContentTypeBytePrefix::try_from(*first_byte)?;
                let remaining_bytes = &data[1..];
                match type_prefix {
                    ContentTypeBytePrefix::Nil => {
                        return Ok((UnitContent::Nil, 1));
                    }
                    ContentTypeBytePrefix::Boolean => match remaining_bytes.get(0) {
                        None => return Err(UnitContentError::MissingDataBytes.into()),
                        Some(data_byte) => {
                            return Ok((UnitContent::Bool(u8_to_bool(*data_byte)), 2));
                        }
                    },
                    ContentTypeBytePrefix::Float64 => {
                        if remaining_bytes.len() < 8 {
                            return Err(UnitContentError::MissingDataBytes.into());
                        } else {
                            let array: [u8; 8] = [
                                remaining_bytes[0],
                                remaining_bytes[1],
                                remaining_bytes[2],
                                remaining_bytes[3],
                                remaining_bytes[4],
                                remaining_bytes[5],
                                remaining_bytes[6],
                                remaining_bytes[7],
                            ];
                            return Ok((UnitContent::Float64(u8_array_to_f64(&array)), 9));
                        }
                    }
                    ContentTypeBytePrefix::Bytes => {
                        let (length, offset) = varint_decode(&remaining_bytes)
                            .map_err(|_| UnitContentError::UnexpectedLengthBytes)?;
                        return Ok((
                            UnitContent::Bytes(
                                remaining_bytes[offset..offset + length as usize].to_vec(),
                            ),
                            1 + length as usize,
                        ));
                    }
                    ContentTypeBytePrefix::JsonString => {
                        let (length, offset) = varint_decode(&remaining_bytes)
                            .map_err(|_| UnitContentError::UnexpectedLengthBytes)?;
                        let string_bytes = &remaining_bytes[offset..offset + length as usize];
                        return Ok((
                            UnitContent::JsonString(utf8_to_string(string_bytes)),
                            1 + length as usize,
                        ));
                    }
                    ContentTypeBytePrefix::BsonBytes => {
                        let (length, offset) = varint_decode(&remaining_bytes)
                            .map_err(|_| UnitContentError::UnexpectedLengthBytes)?;
                        return Ok((
                            UnitContent::BsonBytes(
                                remaining_bytes[offset..offset + length as usize].to_vec(),
                            ),
                            1 + length as usize,
                        ));
                    }
                    ContentTypeBytePrefix::String => {
                        let (length, offset) = varint_decode(&remaining_bytes)
                            .map_err(|_| UnitContentError::UnexpectedLengthBytes)?;
                        let string_bytes = &remaining_bytes[offset..offset + length as usize];
                        return Ok((
                            UnitContent::String(utf8_to_string(string_bytes)),
                            1 + length as usize,
                        ));
                    }
                }
            }
        }
    }
    pub fn parse_data(data: &[u8]) -> ImmuxResult<(Self)> {
        Self::parse(data).map(|(content, _offset)| content)
    }
}

impl PartialEq<JsonValue> for UnitContent {
    fn eq(&self, other: &JsonValue) -> bool {
        match other {
            JsonValue::Array(_) => false,
            JsonValue::Object(_) => false,
            JsonValue::Bool(bool_json) => match self {
                UnitContent::Bool(bool_content) => bool_content == bool_json,
                _ => false,
            },
            JsonValue::Number(n_json) => match self {
                UnitContent::Float64(f_content) => Some(*f_content) == n_json.as_f64(),
                _ => false,
            },
            JsonValue::Null => match self {
                UnitContent::Nil => true,
                _ => false,
            },
            JsonValue::String(s_json) => match self {
                UnitContent::String(s_content) => s_content == s_json,
                _ => false,
            },
        }
    }
}

impl Display for UnitContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        let unit_content_prefix = get_content_type_str_prefix(&self);

        match self {
            UnitContent::Nil => write!(
                f,
                "{}{}{}",
                unit_content_prefix,
                CONTENT_TYPE_VALUE_SEPARATOR,
                String::from("nil")
            ),
            UnitContent::String(string) => write!(
                f,
                "{}{}{}",
                unit_content_prefix,
                CONTENT_TYPE_VALUE_SEPARATOR,
                string.clone()
            ),
            UnitContent::JsonString(string) => write!(
                f,
                "{}{}{}",
                unit_content_prefix,
                CONTENT_TYPE_VALUE_SEPARATOR,
                string.clone()
            ),
            UnitContent::Float64(number) => write!(
                f,
                "{}{}{}",
                unit_content_prefix, CONTENT_TYPE_VALUE_SEPARATOR, number
            ),
            UnitContent::Bool(b) => write!(
                f,
                "{}{}{}",
                unit_content_prefix, CONTENT_TYPE_VALUE_SEPARATOR, b
            ),
            UnitContent::Bytes(bytes) => {
                let bytes_str = encode_hex(bytes)?;
                return write!(
                    f,
                    "{}{}[{}]",
                    unit_content_prefix, CONTENT_TYPE_VALUE_SEPARATOR, bytes_str
                );
            }
            UnitContent::BsonBytes(bytes) => {
                let bytes_str = encode_hex(bytes)?;
                return write!(
                    f,
                    "{}{}[{}]",
                    unit_content_prefix, CONTENT_TYPE_VALUE_SEPARATOR, bytes_str
                );
            }
        }
    }
}

impl FromStr for UnitContent {
    type Err = UnitContentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split(CONTENT_TYPE_VALUE_SEPARATOR).collect();
        if let Some(content_type_str) = data.get(0) {
            match content_type_str {
                &CONTENT_NIL_TYPE_STR_PREFIX => {
                    return Ok(UnitContent::Nil);
                }
                &CONTENT_STRING_TYPE_STR_PREFIX => {
                    return Ok(UnitContent::String(
                        data[1..].join(CONTENT_TYPE_VALUE_SEPARATOR),
                    ));
                }
                &CONTENT_JSON_STRING_TYPE_STR_PREFIX => {
                    return Ok(UnitContent::JsonString(
                        data[1..].join(CONTENT_TYPE_VALUE_SEPARATOR),
                    ));
                }
                &CONTENT_FLOAT64_TYPE_STR_PREFIX => {
                    let f64_str = data[1..].join(CONTENT_TYPE_VALUE_SEPARATOR);
                    if let Ok(f64_num) = f64_str.parse() {
                        return Ok(UnitContent::Float64(f64_num));
                    } else {
                        return Err(UnitContentError::ParseStringError);
                    }
                }
                &CONTENT_BOOL_TYPE_STR_PREFIX => {
                    let bool_str: &str = &data[1..].join(CONTENT_TYPE_VALUE_SEPARATOR);
                    if let Ok(bool) = bool_str.parse() {
                        return Ok(UnitContent::Bool(bool));
                    } else {
                        Err(UnitContentError::ParseStringError)
                    }
                }
                &CONTENT_BYTES_TYPE_STR_PREFIX => {
                    if let Ok(bytes) = decode_hex(&data[1..].join(CONTENT_TYPE_VALUE_SEPARATOR)) {
                        return Ok(UnitContent::Bytes(bytes));
                    } else {
                        Err(UnitContentError::ParseStringError)
                    }
                }
                &CONTENT_BSON_BYTES_TYPE_STR_PREFIX => {
                    if let Ok(bytes) = decode_hex(&data[1..].join(CONTENT_TYPE_VALUE_SEPARATOR)) {
                        return Ok(UnitContent::BsonBytes(bytes));
                    } else {
                        Err(UnitContentError::ParseStringError)
                    }
                }
                _ => {
                    return Err(UnitContentError::UnexpectedTypeStringPrefix);
                }
            }
        } else {
            return Err(UnitContentError::UnexpectedTypeStringPrefix);
        }
    }
}

#[cfg(test)]
mod unit_content_tests {
    use std::str::FromStr;

    use serde_json::Value as JsonValue;

    use crate::declarations::basics::UnitContent;

    fn get_fixture() -> Vec<(Option<UnitContent>, Vec<u8>)> {
        vec![
            (Some(UnitContent::Nil), vec![0x00]),
            (Some(UnitContent::Bool(true)), vec![0x11, 0x01]),
            (Some(UnitContent::Bool(false)), vec![0x11, 0x00]),
            (
                Some(UnitContent::Float64(1.5)),
                vec![0x12, 0, 0, 0, 0, 0, 0, 0xf8, 0x3f],
            ),
            (
                Some(UnitContent::String(String::from("hello"))),
                vec![0x10, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f],
            ),
            (
                Some(UnitContent::JsonString(String::from(
                    r#"{"f64": 0.0, "str": "string_0", "bool": true}"#,
                ))),
                vec![
                    0x21, // type
                    0x2d, // length
                    0x7b, 0x22, 0x66, 0x36, 0x34, 0x22, 0x3a, 0x20, 0x30, 0x2e, 0x30, 0x2c, 0x20,
                    0x22, 0x73, 0x74, 0x72, 0x22, 0x3a, 0x20, 0x22, 0x73, 0x74, 0x72, 0x69, 0x6e,
                    0x67, 0x5f, 0x30, 0x22, 0x2c, 0x20, 0x22, 0x62, 0x6f, 0x6f, 0x6c, 0x22, 0x3a,
                    0x20, 0x74, 0x72, 0x75, 0x65, 0x7d,
                ],
            ),
            (
                Some(UnitContent::BsonBytes(vec![0x01, 0x02, 0x03])),
                vec![0x22, 0x03, 0x01, 0x02, 0x03],
            ),
            (
                Some(UnitContent::Bytes(
                    vec![0].into_iter().cycle().take(255).collect(),
                )),
                vec![
                    0xff, 0xfd, 0xff, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0,
                ],
            ),
            // Non-existing type
            (None, vec![0xaa, 0x01, 0x02, 0x03]),
            // Malformed boolean
            (None, vec![0x11]),
            // Malformed bytes with wrong varint length
            (None, vec![0xff, 0xff, 0x10]),
            // Empty input
            (None, vec![]),
        ]
    }

    #[test]
    fn test_serialize() {
        let table = get_fixture();
        assert!(table.len() > 0);
        for row in table {
            if let (Some(content), expected) = row {
                let serialized = content.marshal();
                assert_eq!(expected, serialized);
            } else {
                // Malformed bytes, skip
            }
        }
    }

    #[test]
    fn test_deserialize() {
        let table = get_fixture();
        assert!(table.len() > 0);
        for row in table {
            let (expected, data) = row;
            if let Some(expected_content) = expected {
                let parsed = UnitContent::parse_data(&data).unwrap();
                assert_eq!(expected_content, parsed);
            } else {
                match UnitContent::parse(&data) {
                    Ok(_) => panic!("Should not be able to parse {:?}", data),
                    Err(_) => (),
                }
            }
        }
    }

    #[test]
    fn test_partial_eq_json_null() {
        let json_null = JsonValue::Null;
        let content_null = UnitContent::Nil;
        let content_bool = UnitContent::Bool(false);
        assert_eq!(content_null, json_null);
        assert_ne!(content_bool, json_null);
    }

    #[test]
    fn test_partial_eq_json_string() {
        let string = String::from("string value");
        let json_string = JsonValue::String(string.clone());
        let json_string_alternative = JsonValue::String("another string value".to_string());
        let content_string = UnitContent::String(string.clone());
        assert_eq!(content_string, json_string);
        assert_ne!(content_string, json_string_alternative);
    }

    #[test]
    fn test_partial_eq_float() {
        let float = -3.14f64;
        let json_float = JsonValue::from(float);
        let json_float_alternative = JsonValue::from(0.1);
        let content_number = UnitContent::Float64(float);
        assert_eq!(content_number, json_float);
        assert_ne!(content_number, json_float_alternative);
    }

    #[test]
    fn test_partial_eq_int() {
        let int = -3i8;
        let json_int = JsonValue::from(int);
        let json_int_alternative = JsonValue::from(100u8);
        let content_number = UnitContent::Float64(int.into());
        assert_eq!(content_number, json_int);
        assert_ne!(content_number, json_int_alternative);
    }

    #[test]
    fn test_partial_eq_bool() {
        assert_eq!(UnitContent::Bool(true), JsonValue::from(true));
        assert_eq!(UnitContent::Bool(false), JsonValue::from(false));
        assert_ne!(UnitContent::Bool(true), JsonValue::from(false));
        assert_ne!(UnitContent::Bool(false), JsonValue::from(true));
    }

    #[test]
    fn test_from_str_and_to_str() {
        let strs = [
            "_nil#nil",
            "_str#hello#world",
            "_bool#true",
            "_bool#false",
            "_f64#1.534",
            r#"_json_str#{"f64": 0.0, "str": "string_0", "bool": true}"#,
            "_bson_bytes#[0x01,0x02,0x03,0xff,0x3f,0x00]",
            "_bytes#[0x7b,0x22,0x66,0x36,0x34,0x00,0xff]",
        ];

        let unit_contents = [
            UnitContent::Nil,
            UnitContent::String("hello#world".to_string()),
            UnitContent::Bool(true),
            UnitContent::Bool(false),
            UnitContent::Float64(1.534),
            UnitContent::JsonString(r#"{"f64": 0.0, "str": "string_0", "bool": true}"#.to_string()),
            UnitContent::BsonBytes(vec![0x01, 0x02, 0x03, 0xff, 0x3f, 0x00]),
            UnitContent::Bytes(vec![0x7b, 0x22, 0x66, 0x36, 0x34, 0x00, 0xff]),
        ];

        for (index, string) in strs.iter().enumerate() {
            let expected_data = string;
            let actual_data = &unit_contents[index].to_string();
            assert_eq!(expected_data, actual_data);

            let expected_data = &unit_contents[index];
            let actual_data = &UnitContent::from_str(string).unwrap();
            assert_eq!(expected_data, actual_data);

            assert_eq!(
                string,
                &(UnitContent::from_str(string)).unwrap().to_string()
            );
            assert_eq!(
                unit_contents[index],
                UnitContent::from_str(&unit_contents[index].to_string()).unwrap()
            );
        }
    }
}
