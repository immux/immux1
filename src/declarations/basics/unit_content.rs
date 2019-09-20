use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::declarations::errors::ImmuxResult;
use crate::utils::{
    bool_to_u8, f64_to_u8_array, u8_array_to_f64, u8_to_bool, utf8_to_string, varint_decode,
    varint_encode,
};

#[repr(u8)]
pub enum ContentTypePrefix {
    Nil = 0x00,

    String = 0x10,
    Boolean = 0x11,
    Float64 = 0x12,

    JsonString = 0x21,
    BsonBytes = 0x22,

    Bytes = 0xff,
}

impl TryFrom<u8> for ContentTypePrefix {
    type Error = UnitContentError;
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        if byte == ContentTypePrefix::Nil as u8 {
            return Ok(ContentTypePrefix::Nil);
        } else if byte == ContentTypePrefix::Bytes as u8 {
            return Ok(ContentTypePrefix::Bytes);
        } else if byte == ContentTypePrefix::JsonString as u8 {
            return Ok(ContentTypePrefix::JsonString);
        } else if byte == ContentTypePrefix::BsonBytes as u8 {
            return Ok(ContentTypePrefix::BsonBytes);
        } else if byte == ContentTypePrefix::String as u8 {
            return Ok(ContentTypePrefix::String);
        } else if byte == ContentTypePrefix::Boolean as u8 {
            return Ok(ContentTypePrefix::Boolean);
        } else if byte == ContentTypePrefix::Float64 as u8 {
            return Ok(ContentTypePrefix::Float64);
        } else {
            return Err(UnitContentError::UnexpectedTypePrefix(byte));
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
    UnexpectedTypePrefix(u8),
    EmptyInput,
    MissingDataBytes,
    UnexpectedLengthBytes,
}

impl UnitContent {
    pub fn marshal(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(1);
        match &self {
            UnitContent::Nil => result.push(ContentTypePrefix::Nil as u8),
            UnitContent::Bool(boolean) => {
                result.push(ContentTypePrefix::Boolean as u8);
                result.push(bool_to_u8(*boolean));
            }
            UnitContent::Float64(number_f64) => {
                result.push(ContentTypePrefix::Float64 as u8);
                result.extend_from_slice(&f64_to_u8_array(*number_f64));
            }
            UnitContent::Bytes(bytes) => {
                result.push(ContentTypePrefix::Bytes as u8);
                result.extend_from_slice(&varint_encode(bytes.len() as u64));
                result.extend_from_slice(bytes)
            }
            UnitContent::JsonString(string) => {
                let bytes = string.as_bytes();
                result.push(ContentTypePrefix::JsonString as u8);
                result.extend_from_slice(&varint_encode(bytes.len() as u64));
                result.extend_from_slice(bytes)
            }
            UnitContent::BsonBytes(bytes) => {
                result.push(ContentTypePrefix::BsonBytes as u8);
                result.extend_from_slice(&varint_encode(bytes.len() as u64));
                result.extend_from_slice(bytes)
            }
            UnitContent::String(string) => {
                let bytes = string.as_bytes();
                result.push(ContentTypePrefix::String as u8);
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
                let type_prefix = ContentTypePrefix::try_from(*first_byte)?;
                let remaining_bytes = &data[1..];
                match type_prefix {
                    ContentTypePrefix::Nil => {
                        return Ok((UnitContent::Nil, 1));
                    }
                    ContentTypePrefix::Boolean => match remaining_bytes.get(0) {
                        None => return Err(UnitContentError::MissingDataBytes.into()),
                        Some(data_byte) => {
                            return Ok((UnitContent::Bool(u8_to_bool(*data_byte)), 2));
                        }
                    },
                    ContentTypePrefix::Float64 => {
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
                    ContentTypePrefix::Bytes => {
                        let (length, offset) = varint_decode(&remaining_bytes)
                            .map_err(|_| UnitContentError::UnexpectedLengthBytes)?;
                        return Ok((
                            UnitContent::Bytes(
                                remaining_bytes[offset..offset + length as usize].to_vec(),
                            ),
                            1 + length as usize,
                        ));
                    }
                    ContentTypePrefix::JsonString => {
                        let (length, offset) = varint_decode(&remaining_bytes)
                            .map_err(|_| UnitContentError::UnexpectedLengthBytes)?;
                        let string_bytes = &remaining_bytes[offset..offset + length as usize];
                        return Ok((
                            UnitContent::JsonString(utf8_to_string(string_bytes)),
                            1 + length as usize,
                        ));
                    }
                    ContentTypePrefix::BsonBytes => {
                        let (length, offset) = varint_decode(&remaining_bytes)
                            .map_err(|_| UnitContentError::UnexpectedLengthBytes)?;
                        return Ok((
                            UnitContent::BsonBytes(
                                remaining_bytes[offset..offset + length as usize].to_vec(),
                            ),
                            1 + length as usize,
                        ));
                    }
                    ContentTypePrefix::String => {
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

impl ToString for UnitContent {
    fn to_string(&self) -> String {
        match self {
            UnitContent::Nil => String::from("nil"),
            UnitContent::String(string) => string.clone(),
            UnitContent::JsonString(string) => string.clone(),
            UnitContent::Float64(f) => format!("{}", f),
            UnitContent::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
            UnitContent::Bytes(bytes) => utf8_to_string(bytes),
            UnitContent::BsonBytes(bytes) => utf8_to_string(bytes),
        }
    }
}

#[cfg(test)]
mod unit_content_tests {
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

}
