use serde::{Deserialize, Serialize};

use crate::declarations::errors::ImmuxError;
use crate::utils::{varint_decode, varint_encode};

#[derive(Debug)]
pub enum ChainHeightError {
    UnexpectedLength(usize),
    ParseError,
}

impl From<ChainHeightError> for ImmuxError {
    fn from(error: ChainHeightError) -> ImmuxError {
        ImmuxError::ChainHeight(error)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChainHeight(u64);

impl ChainHeight {
    pub fn new(data: u64) -> Self {
        Self(data)
    }
    pub fn decrement(&mut self) -> () {
        self.0 -= 1;
    }
    pub fn increment(&mut self) -> () {
        self.0 += 1;
    }
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    pub fn marshal(&self) -> Vec<u8> {
        varint_encode(self.as_u64())
    }
    pub fn parse(data: &[u8]) -> Result<(Self, usize), ChainHeightError> {
        match varint_decode(data) {
            Err(_) => Err(ChainHeightError::ParseError),
            Ok((value, width)) => Ok((ChainHeight::new(value), width)),
        }
    }
}
