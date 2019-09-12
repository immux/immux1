use crate::storage::vkv::chain_height::ChainHeightError;
use crate::storage::vkv::ChainHeight;
use crate::utils::{varint_decode, varint_encode};

#[derive(Debug, Clone, PartialEq)]
pub struct HeightList(Vec<u8>);

impl HeightList {
    pub fn new(data: &[ChainHeight]) -> Self {
        let bytes: Vec<u8> = data
            .iter()
            .map(|height| height.marshal())
            .flatten()
            .collect();
        Self(bytes)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn push(&mut self, height: ChainHeight) {
        self.0.extend(height.marshal())
    }

    pub fn marshal(&self) -> Vec<u8> {
        let data = self.as_bytes();
        let length_bytes = varint_encode(data.len() as u64);

        let mut result = Vec::with_capacity(length_bytes.len() + data.len());
        result.extend(length_bytes);
        result.extend_from_slice(data);
        result
    }

    pub fn parse(data: &[u8]) -> Result<Self, ChainHeightError> {
        let (length, offset) = varint_decode(data).map_err(|_| ChainHeightError::ParseError)?;
        let expected_end = offset + length as usize;
        if data.len() < expected_end {
            return Err(ChainHeightError::UnexpectedLength(data.len()));
        } else {
            return Ok(HeightList(data[offset..expected_end].to_vec()));
        }
    }

    pub fn iter(&self) -> HeightListIterator {
        HeightListIterator::new(self)
    }
}

pub struct HeightListIterator<'a> {
    data: &'a [u8],
    remaining_index: usize,
}

impl<'a> HeightListIterator<'a> {
    fn new(list: &'a HeightList) -> Self {
        HeightListIterator {
            data: list.as_bytes(),
            remaining_index: 0,
        }
    }
}

impl<'a> Iterator for HeightListIterator<'a> {
    type Item = ChainHeight;
    fn next(&mut self) -> Option<ChainHeight> {
        match ChainHeight::parse(&self.data[self.remaining_index..]) {
            Err(_) => None,
            Ok((value, width)) => {
                self.remaining_index += width;
                Some(value)
            }
        }
    }
}

#[cfg(test)]
mod height_list_tests {
    use crate::storage::vkv::height_list::HeightList;
    use crate::storage::vkv::ChainHeight;

    #[test]
    fn test_serialize_data() {
        let list = HeightList::new(&[
            ChainHeight::new(0x12345678),
            ChainHeight::new(0),
            ChainHeight::new(0xff),
        ]);
        let serialized = list.marshal();
        let expected = vec![
            0x09, // data length
            0xfe, 0x78, 0x56, 0x34, 0x12, // height 1,
            0x00, // height 2,
            0xfd, 0xff, 0x00, // height 3
        ];
        assert_eq!(serialized, expected)
    }

    #[test]
    fn test_parse_data() {
        let data = [
            0x09, // data length
            0xfe, 0x78, 0x56, 0x34, 0x12, // height 1,
            0x00, // height 2,
            0xfd, 0xff, 0x00, // height 3
        ];
        let parsed = HeightList::parse(&data).unwrap();
        let expected = HeightList::new(&[
            ChainHeight::new(0x12345678),
            ChainHeight::new(0),
            ChainHeight::new(0xff),
        ]);
        assert_eq!(parsed, expected)
    }

    #[test]
    fn test_parse_empty() {
        let data = [0x00];
        let parsed = HeightList::parse(&data).unwrap();
        let expected = HeightList::new(&[]);
        assert_eq!(parsed, expected)
    }

    #[test]
    fn test_parsing_reversibility() {
        for i in 1..100u64 {
            let heights: Vec<ChainHeight> = [i]
                .iter()
                .enumerate()
                .cycle()
                .take(i as usize)
                .map(|(index, i)| ChainHeight::new(*i + index as u64))
                .collect();
            let list = HeightList::new(&heights);
            let serialized = list.marshal();
            let parsed = HeightList::parse(&serialized).unwrap();
            assert_eq!(list, parsed)
        }
    }

    #[test]
    fn test_serialize_empty() {
        let list = HeightList::new(&[]);
        let serialized = list.marshal();
        let expected = vec![0x00];
        assert_eq!(serialized, expected)
    }

    #[test]
    fn test_internal_byte_representation() {
        let list = HeightList::new(&[
            ChainHeight::new(0x11223344),
            ChainHeight::new(0x10),
            ChainHeight::new(0xffff),
        ]);
        let expected_bytes = [
            0xfe, 0x44, 0x33, 0x22, 0x11, // first
            0x10, // second
            0xfd, 0xff, 0xff, // third
        ];
        assert_eq!(list.as_bytes(), expected_bytes)
    }

    #[test]
    fn test_iterator() {
        let heights: Vec<_> = [0, 1, 0xffff, 0x1234568]
            .iter()
            .map(|h| ChainHeight::new(*h))
            .collect();
        let list = HeightList::new(&heights);
        let heights_from_list: Vec<ChainHeight> = list.iter().collect();
        assert_eq!(heights, heights_from_list);
    }

    #[test]
    fn test_push() {
        let mut list = HeightList::new(&[ChainHeight::new(0x01)]);
        list.push(ChainHeight::new(0x02));
        let expected = HeightList::new(&[ChainHeight::new(0x01), ChainHeight::new(0x02)]);
        assert_eq!(list, expected)
    }
}
