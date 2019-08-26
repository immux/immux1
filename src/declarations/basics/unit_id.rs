use std::convert::{From, TryFrom};

use serde::{Deserialize, Serialize};

use crate::declarations::errors::ImmuxError;
use crate::utils::{u128_to_u8_array, u8_array_to_u128};

pub const UNIT_ID_BYTES: usize = 16;

#[derive(Debug)]
pub enum UnitIdError {
    InsufficientLength(Vec<u8>),
    CannotParseString(String),
}

impl From<UnitIdError> for ImmuxError {
    fn from(error: UnitIdError) -> Self {
        ImmuxError::UnitId(error)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub struct UnitId(u128);

impl UnitId {
    pub fn new(id: u128) -> Self {
        Self(id)
    }
    pub fn marshal(&self) -> Vec<u8> {
        u128_to_u8_array(self.0).to_vec()
    }
    pub fn as_int(&self) -> u128 {
        self.0
    }
}

impl From<&[u8; 16]> for UnitId {
    fn from(data: &[u8; UNIT_ID_BYTES]) -> Self {
        UnitId::new(u8_array_to_u128(data))
    }
}

impl TryFrom<Vec<u8>> for UnitId {
    type Error = UnitIdError;
    fn try_from(data: Vec<u8>) -> Result<UnitId, UnitIdError> {
        if data.len() < UNIT_ID_BYTES {
            Err(UnitIdError::InsufficientLength(data))
        } else {
            let array: [u8; UNIT_ID_BYTES] = [
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
                data[9], data[10], data[11], data[12], data[13], data[14], data[15],
            ];
            Ok(UnitId::from(&array))
        }
    }
}

impl TryFrom<&str> for UnitId {
    type Error = UnitIdError;
    fn try_from(data: &str) -> Result<UnitId, UnitIdError> {
        match data.parse::<u128>() {
            Err(_) => Err(UnitIdError::CannotParseString(data.to_owned())),
            Ok(u) => Ok(UnitId::new(u)),
        }
    }
}
