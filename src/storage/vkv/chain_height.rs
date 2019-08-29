use serde::{Deserialize, Serialize};

use crate::utils::{u64_to_u8_array, u8_array_to_u64};

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
}

impl Into<u64> for ChainHeight {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<[u8; 8]> for ChainHeight {
    fn into(self) -> [u8; 8] {
        u64_to_u8_array(self.into())
    }
}

impl Into<Vec<u8>> for ChainHeight {
    fn into(self) -> Vec<u8> {
        u64_to_u8_array(self.0).to_vec()
    }
}

impl From<[u8; 8]> for ChainHeight {
    fn from(data: [u8; 8]) -> Self {
        Self(u8_array_to_u64(&data))
    }
}
