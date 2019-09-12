use serde::{Deserialize, Serialize};

use crate::declarations::errors::ImmuxError;
use crate::utils::{varint_decode, varint_encode, VarIntError};

const NONE_VALUE_MARKER: u8 = 0x00;
const EXTANT_VALUE_MARKER: u8 = 0xff;

#[derive(Debug)]
pub enum StoreValueError {
    InsufficientBytes,
    IncorrectLengthFormat,
}

impl From<StoreValueError> for ImmuxError {
    fn from(error: StoreValueError) -> ImmuxError {
        ImmuxError::StoreValue(error)
    }
}

impl From<VarIntError> for StoreValueError {
    fn from(error: VarIntError) -> StoreValueError {
        match error {
            VarIntError::UnexpectedFormat => StoreValueError::IncorrectLengthFormat,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StoreValue(Option<Vec<u8>>);

impl StoreValue {
    pub fn new(data: Option<Vec<u8>>) -> Self {
        Self(data)
    }
    pub fn inner(&self) -> &Option<Vec<u8>> {
        &self.0
    }
    pub fn marshal(&self) -> Vec<u8> {
        match self.inner() {
            None => vec![NONE_VALUE_MARKER],
            Some(data) => {
                let mut result = Vec::new();
                result.push(EXTANT_VALUE_MARKER);
                result.extend(varint_encode(data.len() as u64));
                result.extend_from_slice(data);
                return result;
            }
        }
    }
    pub fn parse(data: &[u8]) -> Result<(StoreValue, usize), StoreValueError> {
        match data.get(0) {
            None => return Err(StoreValueError::InsufficientBytes),
            Some(&NONE_VALUE_MARKER) => return Ok((StoreValue::new(None), 1)),
            Some(_) => {
                let (value_length, width) = varint_decode(&data[1..])?;
                let expected_full_length = 1 + width + value_length as usize;
                if data.len() < expected_full_length {
                    return Err(StoreValueError::InsufficientBytes);
                }
                Ok((
                    StoreValue::new(Some(data[1 + width..expected_full_length].to_vec())),
                    expected_full_length,
                ))
            }
        }
    }
}

#[cfg(test)]
mod store_value_tests {
    use crate::declarations::basics::StoreValue;

    #[test]
    fn test_store_value_serialize_none() {
        let value = StoreValue::new(None);
        assert_eq!(value.marshal(), vec![0x00]);
    }

    #[test]
    fn test_store_value_parse_none() {
        let data = vec![0x00];
        let (value, width) = StoreValue::parse(&data).unwrap();
        assert_eq!(value, StoreValue::new(None));
        assert_eq!(width, 1);
    }

    #[test]
    fn test_store_value_serialize_data() {
        let value = StoreValue::new(Some(vec![0x10, 0x20, 0x30]));
        assert_eq!(value.marshal(), vec![0xff, 0x03, 0x10, 0x20, 0x30]);
    }

    #[test]
    fn test_store_value_parse_data() {
        let data = vec![0xff, 0x02, 0x00, 0x01];
        let (value, width) = StoreValue::parse(&data).unwrap();
        assert_eq!(value, StoreValue::new(Some(vec![0x00, 0x01])));
        assert_eq!(width, 4);
    }

    #[test]
    fn test_store_value_parse_data_width_excess_bytes() {
        let data = vec![0xff, 0x02, 0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff];
        let (value, width) = StoreValue::parse(&data).unwrap();
        assert_eq!(value, StoreValue::new(Some(vec![0x00, 0x01])));
        assert_eq!(width, 4);
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoxedStoreValue(Option<Box<[u8]>>);

impl BoxedStoreValue {
    pub fn new(data: Option<Vec<u8>>) -> Self {
        BoxedStoreValue(data.map(|vector| vector.into_boxed_slice()))
    }
    pub fn inner(&self) -> &Option<Box<[u8]>> {
        &self.0
    }
}

impl From<StoreValue> for BoxedStoreValue {
    fn from(value: StoreValue) -> BoxedStoreValue {
        BoxedStoreValue::new(value.inner().to_owned())
    }
}
