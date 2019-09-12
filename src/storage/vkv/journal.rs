use crate::declarations::basics::StoreValue;
use crate::declarations::errors::ImmuxResult;
use crate::storage::vkv::height_list::HeightList;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitJournal {
    pub value: StoreValue,
    pub update_heights: HeightList,
}

impl UnitJournal {
    pub fn marshal(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.value.marshal());
        result.extend(self.update_heights.marshal());
        return result;
    }
    pub fn parse(data: &[u8]) -> ImmuxResult<Self> {
        let (value, value_width) = StoreValue::parse(&data)?;
        let update_heights = HeightList::parse(&data[value_width..])?;
        return Ok(UnitJournal {
            value,
            update_heights,
        });
    }
}

#[cfg(test)]
mod journal_tests {
    use crate::declarations::basics::StoreValue;
    use crate::storage::vkv::height_list::HeightList;
    use crate::storage::vkv::journal::UnitJournal;
    use crate::storage::vkv::ChainHeight;

    #[test]
    fn test_journal_serialize() {
        let journal = UnitJournal {
            value: StoreValue::new(Some(vec![1, 2, 3])),
            update_heights: HeightList::new(&[
                ChainHeight::new(0),
                ChainHeight::new(0xf0),
                ChainHeight::new(0xff00),
            ]),
        };
        let serialized = journal.marshal();
        let expected = vec![
            // value
            0xff, // Sigil
            0x03, // StoreValue length
            1, 2, 3, // StoreValue
            // update_heights
            0x05, // height length
            0,    // height 0
            0xf0, // height 1
            0xfd, 0x00, 0xff, // height 2
        ];
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_journal_parse() {
        let data = vec![
            // value
            0xff, // Sigil
            0x01, // StoreValue length
            0xaa, // StoreValue
            // update_heights
            0x04, // height length
            0xf0, // height 0
            0xfd, 0x00, 0xff, // height 1
        ];
        let parsed = UnitJournal::parse(&data).unwrap();
        let expected = UnitJournal {
            value: StoreValue::new(Some(vec![0xaa])),
            update_heights: HeightList::new(&[ChainHeight::new(0xf0), ChainHeight::new(0xff00)]),
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_journal_serialize_reversibility() {
        let values = [None, Some(vec![]), Some(vec![1, 2, 3]), Some(vec![0xff])];
        let store_values: Vec<_> = values
            .iter()
            .map(|v| StoreValue::new(v.to_owned()))
            .collect();

        for store_value in &store_values {
            for i in 0..10 {
                let heights: Vec<ChainHeight> = (1..i).map(|i| ChainHeight::new(i)).collect();
                let journal = UnitJournal {
                    value: store_value.to_owned(),
                    update_heights: HeightList::new(&heights),
                };
                let serialized = journal.marshal();
                let parsed = UnitJournal::parse(&serialized).unwrap();
                assert_eq!(journal, parsed)
            }
        }
    }

}
