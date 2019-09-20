use std::vec::IntoIter as VecIntoIter;

use serde::{Deserialize, Serialize};

use crate::config::MAX_PROPERTY_NAME_LENGTH;
use crate::declarations::errors::ImmuxError;
use crate::utils::utf8_to_string;

#[derive(Debug)]
pub enum PropertyNameListError {
    CannotParse,
}

impl From<PropertyNameListError> for ImmuxError {
    fn from(error: PropertyNameListError) -> ImmuxError {
        ImmuxError::PropertyName(error)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PropertyName(Vec<u8>);

impl PropertyName {
    pub fn new(bytes: &[u8]) -> Self {
        if bytes.len() < MAX_PROPERTY_NAME_LENGTH {
            Self(bytes.to_vec())
        } else {
            Self(bytes[0..MAX_PROPERTY_NAME_LENGTH].to_vec())
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn marshal(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(self.as_bytes().len() as u8);
        result.extend_from_slice(self.as_bytes());
        result
    }
}

impl ToString for PropertyName {
    fn to_string(&self) -> String {
        utf8_to_string(&self.0)
    }
}

impl From<&str> for PropertyName {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<PropertyName> for Vec<u8> {
    fn from(data: PropertyName) -> Vec<u8> {
        return data.as_bytes().to_vec();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropertyNameList(Vec<PropertyName>);

impl PropertyNameList {
    pub fn new(names: Vec<PropertyName>) -> Self {
        Self(names)
    }

    fn dedup(&mut self) -> () {
        self.0.sort_by(|v1, v2| v1.cmp(v2));
        self.0.dedup_by(|v1, v2| v1 == v2);
    }

    pub fn add(&mut self, data: PropertyName) {
        self.0.push(data);
        self.dedup()
    }

    pub fn as_slice(&self) -> &[PropertyName] {
        &self.0
    }
}

impl IntoIterator for PropertyNameList {
    type Item = PropertyName;
    type IntoIter = VecIntoIter<PropertyName>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod property_names_tests {
    use super::PropertyName;

    #[test]
    fn test_creation() {
        let name = PropertyName::new(&[1, 2, 3]);
        assert_eq!(name.as_bytes(), &[1, 2, 3])
    }

    #[test]
    fn test_from_str() {
        let name = PropertyName::from("abc");
        assert_eq!(name.as_bytes(), &[97, 98, 99])
    }

    #[test]
    fn test_to_string() {
        let name = PropertyName::new(&[97, 97, 97]);
        assert_eq!(name.to_string(), "aaa")
    }

    #[test]
    fn test_to_vec() {
        let name = PropertyName::new(&[0, 1, 2, 3]);
        let v: Vec<_> = name.into();
        assert_eq!(v, [0, 1, 2, 3])
    }

    #[test]
    fn test_as_bytes() {
        let name = PropertyName::new(&[0xaa, 0xbb, 0xcc]);
        let serialized = name.as_bytes();
        let expected = &[0xaa, 0xbb, 0xcc];
        assert_eq!(serialized, expected)
    }

    #[test]
    fn test_serialize() {
        let name = PropertyName::new(&[0xaa, 0xbb, 0xcc]);
        let serialized = name.marshal();
        let expected = vec![0x03, 0xaa, 0xbb, 0xcc];
        assert_eq!(serialized, expected)
    }
}

#[cfg(test)]
mod property_name_list_test {
    use super::{PropertyName, PropertyNameList};

    #[test]
    fn test_creation() {
        let name = PropertyName::from("name");
        let list = PropertyNameList::new(vec![name]);
        assert_eq!(list.as_slice().len(), 1);
    }

    #[test]
    fn test_add() {
        let mut list = PropertyNameList::new(vec![]);
        let name_1 = PropertyName::from("1");
        let name_2 = PropertyName::from("2");
        list.add(name_1.clone());
        list.add(name_2.clone());
        list.add(name_1.clone());
        list.add(name_2.clone());
        assert_eq!(list.as_slice().len(), 2);
        assert_eq!(list.as_slice(), &[name_1, name_2]);
    }

    #[test]
    fn test_iterator() {
        let names: Vec<PropertyName> = ["a", "b", "c"]
            .iter()
            .map(|c| PropertyName::from(*c))
            .collect();

        let list = PropertyNameList::new(names.clone());

        for (index, item) in list.into_iter().enumerate() {
            let name = names.get(index).unwrap();
            assert_eq!(name, &item)
        }
    }

}
