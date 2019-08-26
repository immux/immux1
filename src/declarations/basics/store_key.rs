use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::declarations::basics::{GroupingLabel, UnitId, UnitSpecifier};
use crate::declarations::errors::{ImmuxError, ImmuxResult};

#[derive(Debug)]
pub enum StoreKeyError {
    CannotParseToUnitSpecifier,
}

impl From<StoreKeyError> for ImmuxError {
    fn from(error: StoreKeyError) -> ImmuxError {
        ImmuxError::StoreKey(error)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct StoreKey(Vec<u8>);

impl StoreKey {
    pub fn new(data: &[u8]) -> Self {
        StoreKey(data.to_vec())
    }
    pub fn build(grouping: &GroupingLabel, id: UnitId) -> Self {
        let specifier = UnitSpecifier::new(grouping.to_owned(), id);
        Self::from(specifier)
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl From<UnitSpecifier> for StoreKey {
    fn from(specifier: UnitSpecifier) -> Self {
        let (grouping, unit_id) = specifier.into_components();
        let mut result: Vec<u8> = Vec::new();
        result.extend_from_slice(&grouping.marshal());
        result.extend(unit_id.marshal());
        StoreKey(result)
    }
}

impl TryFrom<StoreKey> for UnitSpecifier {
    type Error = ImmuxError;
    fn try_from(store_key: StoreKey) -> ImmuxResult<UnitSpecifier> {
        let data = store_key.0;
        let grouping_length = data[0] as usize;
        if data.len() < 1 + grouping_length {
            return Err(StoreKeyError::CannotParseToUnitSpecifier.into());
        }
        let grouping_bytes = &data[1..1 + grouping_length];
        let grouping = GroupingLabel::from(grouping_bytes.to_owned());
        let unit_id_bytes = &data[1 + grouping_length..];
        match UnitId::try_from(unit_id_bytes.to_vec()) {
            Err(_) => return Err(StoreKeyError::CannotParseToUnitSpecifier.into()),
            Ok(unit_id) => {
                return Ok(UnitSpecifier::new(grouping, unit_id));
            }
        };
    }
}

impl From<StoreKey> for BoxedStoreKey {
    fn from(key: StoreKey) -> Self {
        BoxedStoreKey::new(key.as_slice().to_vec())
    }
}

impl From<BoxedStoreKey> for StoreKey {
    fn from(key: BoxedStoreKey) -> Self {
        StoreKey::new(key.as_slice())
    }
}

impl From<Vec<u8>> for StoreKey {
    fn from(data: Vec<u8>) -> Self {
        StoreKey(data)
    }
}

impl From<&[u8]> for StoreKey {
    fn from(data: &[u8]) -> Self {
        StoreKey(data.to_vec())
    }
}

impl From<String> for StoreKey {
    fn from(data: String) -> Self {
        StoreKey(data.as_bytes().to_vec())
    }
}

pub type StoreKeyFragment = StoreKey;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoxedStoreKey(Box<[u8]>);

impl BoxedStoreKey {
    pub fn new(data: Vec<u8>) -> Self {
        BoxedStoreKey(data.into())
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod store_key_tests {
    use std::convert::TryInto;

    use super::StoreKey;
    use crate::declarations::basics::{GroupingLabel, UnitId, UnitSpecifier};

    #[test]
    fn test_from_unit_specifier() {
        let label = GroupingLabel::from("wow".as_bytes().to_vec());
        let id = UnitId::new(42);
        let specifier = UnitSpecifier::new(label, id);
        let key = StoreKey::from(specifier);
        let bytes: Vec<u8> = key.as_slice().to_vec();
        let expected = [
            3, // group_length
            119, 111, 119, // "wow", grouping
            42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 42, unit key
        ];
        assert_eq!(bytes, expected)
    }

    #[test]
    fn test_to_unit_specifier() {
        let key_data = [
            3, // group_length
            119, 111, 119, // "wow", grouping
            42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 42, unit key
        ];
        let key = StoreKey::new(&key_data);
        let specifier: UnitSpecifier = key.try_into().unwrap();
        let (grouping, id) = specifier.into_components();
        assert_eq!(grouping.as_bytes(), "wow".as_bytes());
        assert_eq!(id, UnitId::new(42));
    }
}
