use std::convert::From;

use serde::{Deserialize, Serialize};

use crate::utils::{u128_to_u8_array, u8_array_to_u128};

pub const UNIT_ID_BYTES: usize = 16;

#[derive(Debug)]
pub enum UnitIdError {
    InsufficientLength(Vec<u8>),
    CannotParseString(String),
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
    pub fn parse(data: &[u8]) -> Result<Self, UnitIdError> {
        if data.len() < UNIT_ID_BYTES {
            Err(UnitIdError::InsufficientLength(data.to_vec()))
        } else {
            let array: [u8; UNIT_ID_BYTES] = [
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
                data[9], data[10], data[11], data[12], data[13], data[14], data[15],
            ];
            Ok(UnitId::from(&array))
        }
    }
    pub fn read_int_in_str(data: &str) -> Result<Self, UnitIdError> {
        match data.parse::<u128>() {
            Err(_) => Err(UnitIdError::CannotParseString(data.to_owned())),
            Ok(u) => Ok(UnitId::new(u)),
        }
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
