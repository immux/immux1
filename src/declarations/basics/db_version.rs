use serde::{Deserialize, Serialize};

use crate::utils::{u32_to_u8_array, u8_array_to_u32};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
pub struct DBVersion(u32);

impl DBVersion {
    /// Creates a new version. Constant function because software version is always hard-coded
    pub const fn new(version: u32) -> DBVersion {
        DBVersion(version)
    }
    pub fn as_int(&self) -> u32 {
        self.0
    }
    pub fn marshal(&self) -> [u8; 4] {
        u32_to_u8_array(self.as_int())
    }
    pub fn parse(data: &[u8; 4]) -> DBVersion {
        DBVersion::new(u8_array_to_u32(data))
    }
}
