use std::convert::TryFrom;
use std::vec::IntoIter as VecIntoIter;

use crate::declarations::basics::{UnitId, UNIT_ID_BYTES};
use crate::declarations::errors::ImmuxError;

#[derive(Debug)]
pub enum IdListError {
    CannotParse,
}

impl From<IdListError> for ImmuxError {
    fn from(error: IdListError) -> Self {
        ImmuxError::IdList(error)
    }
}

#[derive(Clone, Debug)]
pub struct IdList(Vec<UnitId>);

impl IdList {
    pub fn new(list: Vec<UnitId>) -> Self {
        IdList(list)
    }

    pub fn push(&mut self, id: UnitId) -> () {
        self.0.push(id);
        self.dedup();
    }

    pub fn merge(&mut self, new_ids: &IdList) -> () {
        self.0.extend_from_slice(new_ids.as_slice());
        self.dedup();
    }

    pub fn as_slice(&self) -> &[UnitId] {
        &self.0
    }

    fn dedup(&mut self) -> () {
        self.0.sort_by(|v1, v2| v1.cmp(v2));
        self.0.dedup_by(|v1, v2| v1 == v2);
    }

    pub fn marshal(&self) -> Vec<u8> {
        let list = &self.0;
        let mut result: Vec<u8> = Vec::with_capacity(list.len() * UNIT_ID_BYTES);
        for id in list {
            result.extend(id.marshal())
        }
        return result;
    }

    pub fn parse(data: &[u8]) -> Result<Self, IdListError> {
        if data.len() % UNIT_ID_BYTES != 0 {
            return Err(IdListError::CannotParse.into());
        }
        let mut list: Vec<UnitId> = Vec::with_capacity(data.len() / UNIT_ID_BYTES);
        for chunk in data.chunks_exact(UNIT_ID_BYTES) {
            let array: [u8; UNIT_ID_BYTES] = [
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                chunk[8], chunk[9], chunk[10], chunk[11], chunk[12], chunk[13], chunk[14],
                chunk[15],
            ];
            list.push(UnitId::from(&array));
        }
        return Ok(IdList::new(list));
    }
}

impl IntoIterator for IdList {
    type Item = UnitId;
    type IntoIter = VecIntoIter<UnitId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<IdList> for Vec<u8> {
    fn from(ids: IdList) -> Vec<u8> {
        ids.marshal()
    }
}

impl TryFrom<&[u8]> for IdList {
    type Error = IdListError;
    fn try_from(data: &[u8]) -> Result<Self, IdListError> {
        IdList::parse(data)
    }
}

#[cfg(test)]
mod id_list_tests {
    use crate::declarations::basics::{IdList, UnitId};
    use std::convert::TryFrom;

    #[test]
    fn test_serialize_empty() {
        let list = IdList::new(vec![]);
        let serialized = list.marshal();
        let expected: Vec<u8> = vec![];
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_serialize_data() {
        let list = IdList::new(vec![UnitId::new(0), UnitId::new(1)]);
        let serialized = list.marshal();
        let expected: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // first
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // second
        ];
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_parsing_normal() {
        let data: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // first
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // second
        ];
        let parsed = IdList::try_from(data.as_slice()).unwrap();
        let expected = IdList::new(vec![UnitId::new(0), UnitId::new(1)]);
        assert_eq!(parsed.as_slice(), expected.as_slice());
    }

    #[test]
    fn test_parsing_empty() {
        let data: Vec<u8> = vec![];
        let parsed = IdList::try_from(data.as_slice()).unwrap();
        let expected = IdList::new(vec![]);
        assert_eq!(parsed.as_slice(), expected.as_slice());
    }

    #[test]
    #[should_panic]
    fn test_parsing_defective_input() {
        let data: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // first
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // second, defective
        ];
        let parsed = IdList::parse(data.as_slice()).unwrap();
        println!("parsed: {}", parsed.as_slice().len());
    }

    #[test]
    fn test_reversibility() {
        let ids: Vec<UnitId> = (0..255)
            .cycle()
            .map(|i| UnitId::new(i))
            .take(10_000)
            .collect();
        let list = IdList::new(ids.clone());
        let serialized = list.marshal();
        let parsed = IdList::parse(serialized.as_slice()).unwrap();
        assert_eq!(ids.as_slice(), parsed.as_slice())
    }

    #[test]
    fn test_push() {
        let mut list = IdList::new(vec![]);
        list.push(UnitId::new(1));
        list.push(UnitId::new(2));
        list.push(UnitId::new(3));
        list.push(UnitId::new(3));
        list.push(UnitId::new(2));
        list.push(UnitId::new(1));
        assert!(list.as_slice().contains(&UnitId::new(1)));
        assert!(list.as_slice().contains(&UnitId::new(2)));
        assert!(list.as_slice().contains(&UnitId::new(3)));
        assert_eq!(list.as_slice().len(), 3);
    }

    #[test]
    fn test_merge() {
        let mut list = IdList::new(vec![UnitId::new(1), UnitId::new(2), UnitId::new(3)]);
        let list_b = IdList::new(vec![UnitId::new(3), UnitId::new(4), UnitId::new(5)]);
        list.merge(&list_b);

        assert_eq!(list.as_slice().len(), 5);
        assert!(list.as_slice().contains(&UnitId::new(1)));
        assert!(list.as_slice().contains(&UnitId::new(2)));
        assert!(list.as_slice().contains(&UnitId::new(3)));
        assert!(list.as_slice().contains(&UnitId::new(4)));
        assert!(list.as_slice().contains(&UnitId::new(5)));
    }
}
