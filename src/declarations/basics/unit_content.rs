use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::declarations::errors::ImmuxResult;
use crate::utils::{bool_to_u8, f64_to_u8_array, u8_array_to_f64, u8_to_bool, utf8_to_string};

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
}

impl UnitContent {
    pub fn marshal(&self) -> Vec<u8> {
        let prefix_data_pair: (u8, Vec<u8>) = match &self {
            UnitContent::Nil => (ContentTypePrefix::Nil as u8, vec![]),
            UnitContent::Bytes(bytes) => (ContentTypePrefix::Bytes as u8, bytes.to_vec()),
            UnitContent::JsonString(string) => (
                ContentTypePrefix::JsonString as u8,
                string.as_bytes().to_vec(),
            ),
            UnitContent::BsonBytes(bytes) => (ContentTypePrefix::BsonBytes as u8, bytes.to_vec()),
            UnitContent::String(string) => {
                (ContentTypePrefix::String as u8, string.as_bytes().to_vec())
            }
            UnitContent::Bool(boolean) => {
                (ContentTypePrefix::Boolean as u8, vec![bool_to_u8(*boolean)])
            }
            UnitContent::Float64(number_f64) => (
                ContentTypePrefix::Float64 as u8,
                f64_to_u8_array(*number_f64).to_vec(),
            ),
        };
        let mut result = Vec::with_capacity(1 + prefix_data_pair.1.len());
        result.push(prefix_data_pair.0);
        result.extend_from_slice(&prefix_data_pair.1);
        return result;
    }
    pub fn parse(data: &[u8]) -> ImmuxResult<Self> {
        if data.len() == 0 {
            return Err(UnitContentError::EmptyInput.into());
        }
        let type_prefix = ContentTypePrefix::try_from(data[0])?;
        match type_prefix {
            ContentTypePrefix::Nil => {
                return Ok(UnitContent::Nil);
            }
            ContentTypePrefix::Bytes => {
                return Ok(UnitContent::Bytes(data[1..].to_vec()));
            }
            ContentTypePrefix::JsonString => {
                return Ok(UnitContent::JsonString(utf8_to_string(&data[1..])));
            }
            ContentTypePrefix::BsonBytes => {
                return Ok(UnitContent::BsonBytes(data[1..].to_vec()));
            }
            ContentTypePrefix::String => {
                return Ok(UnitContent::String(utf8_to_string(&data[1..])));
            }
            ContentTypePrefix::Boolean => {
                return Ok(UnitContent::Bool(u8_to_bool(data[1])));
            }
            ContentTypePrefix::Float64 => {
                let array: [u8; 8] = [
                    data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
                ];
                return Ok(UnitContent::Float64(u8_array_to_f64(&array)));
            }
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
    use crate::declarations::basics::UnitContent;

    fn get_fixture() -> Vec<(Option<UnitContent>, Vec<u8>)> {
        vec![
            (Some(UnitContent::Nil), vec![0x00]),
            (Some(UnitContent::Bool(true)), vec![0x11, 0x01]),
            (
                Some(UnitContent::Float64(1.5)),
                vec![0x12, 0, 0, 0, 0, 0, 0, 0xf8, 0x3f],
            ),
            (
                Some(UnitContent::String(String::from("hello"))),
                vec![0x10, 0x68, 0x65, 0x6c, 0x6c, 0x6f],
            ),
            (
                Some(UnitContent::JsonString(String::from(
                    r#"{"f64": 0.0, "str": "string_0", "bool": true}"#,
                ))),
                vec![
                    0x21, // type
                    0x7b, 0x22, 0x66, 0x36, 0x34, 0x22, 0x3a, 0x20, 0x30, 0x2e, 0x30, 0x2c, 0x20,
                    0x22, 0x73, 0x74, 0x72, 0x22, 0x3a, 0x20, 0x22, 0x73, 0x74, 0x72, 0x69, 0x6e,
                    0x67, 0x5f, 0x30, 0x22, 0x2c, 0x20, 0x22, 0x62, 0x6f, 0x6f, 0x6c, 0x22, 0x3a,
                    0x20, 0x74, 0x72, 0x75, 0x65, 0x7d,
                ],
            ),
            (
                Some(UnitContent::BsonBytes(vec![0x01, 0x02, 0x03])),
                vec![0x22, 0x01, 0x02, 0x03],
            ),
            (
                Some(UnitContent::Bytes(vec![0x05, 0x06, 0x07])),
                vec![0xff, 0x05, 0x06, 0x07],
            ),
            // Non-existing type
            (None, vec![0xaa, 0x01, 0x02, 0x03]),
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
                let parsed = UnitContent::parse(&data).unwrap();
                assert_eq!(expected_content, parsed);
            } else {
                match UnitContent::parse(&data) {
                    Ok(_) => panic!("Should not be able to parse {:?}", data),
                    Err(_) => (),
                }
            }
        }
    }

}
