use crate::declarations::basics::StoreValue;
use crate::declarations::errors::ImmuxResult;
use crate::storage::vkv::chain_height::ChainHeight;
use crate::storage::vkv::VkvError;
use crate::utils::{bool_to_u8, u32_to_u8_array, u8_array_to_u32, u8_to_bool};

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateRecord {
    pub deleted: bool,
    pub height: ChainHeight,
    pub value: StoreValue,
}

impl UpdateRecord {
    pub fn marshal(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(bool_to_u8(self.deleted));
        let height_bytes: Vec<u8> = self.height.into();
        result.extend(height_bytes);
        let value_bytes = self.value.to_vec();
        result.extend_from_slice(&u32_to_u8_array(value_bytes.len() as u32));
        result.extend(value_bytes);
        return result;
    }
    pub fn parse(data: &[u8]) -> ImmuxResult<Self> {
        if data.len() < 13 {
            return Err(VkvError::UpdateRecordParsing.into());
        }
        let deleted = u8_to_bool(data[0]);
        let height = ChainHeight::from([
            data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
        ]);
        let value_length = u8_array_to_u32(&[data[9], data[10], data[11], data[12]]);
        let expected_end_offset = 13 + value_length as usize;
        if data.len() < expected_end_offset {
            return Err(VkvError::UpdateRecordParsing.into());
        }
        let value_bytes = Vec::from(&data[13..expected_end_offset]);
        let value = StoreValue::from(value_bytes);
        let update_record = UpdateRecord {
            deleted,
            height,
            value,
        };
        return Ok(update_record);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitJournal {
    pub api_version: u32,
    pub updates: Vec<UpdateRecord>,
}

impl UnitJournal {
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&u32_to_u8_array(self.api_version));
        for update in &self.updates {
            let update_serialized = update.marshal();
            result.extend_from_slice(&u32_to_u8_array(update_serialized.len() as u32));
            result.extend_from_slice(&update_serialized);
        }
        return result;
    }
    pub fn parse(data: &[u8]) -> ImmuxResult<Self> {
        if data.len() < 4 {
            return Err(VkvError::JournalParsing.into());
        }
        let api_version = u8_array_to_u32(&[data[0], data[1], data[2], data[3]]);
        let mut i = 4;
        let mut updates: Vec<UpdateRecord> = Vec::new();
        while i < data.len() {
            let length = u8_array_to_u32(&[data[i], data[i + 1], data[i + 2], data[i + 3]]);
            let segment_end = i + 4 + (length as usize);
            if segment_end > data.len() {
                return Err(VkvError::JournalParsing.into());
            }
            let segment = &data[i + 4..segment_end];
            let record = UpdateRecord::parse(segment)?;
            updates.push(record);
            i = segment_end;
        }
        return Ok(UnitJournal {
            api_version,
            updates,
        });
    }
}

#[cfg(test)]
mod update_record_tests {
    use crate::declarations::basics::StoreValue;
    use crate::storage::vkv::chain_height::ChainHeight;
    use crate::storage::vkv::journal::UpdateRecord;

    #[test]
    fn test_update_record_serialize() {
        let record = UpdateRecord {
            deleted: false,
            height: ChainHeight::new(10),
            value: StoreValue::from(vec![10, 20, 30, 40, 50]),
        };
        let serialized = record.marshal();
        let expected = vec![
            0, // deleted
            10, 0, 0, 0, 0, 0, 0, 0, // height
            5, 0, 0, 0, // value length
            10, 20, 30, 40, 50, // value
        ];
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_update_record_deserialize() {
        let data = vec![
            1, // deleted
            200, 0, 0, 0, 0, 0, 0, 0, // height
            3, 0, 0, 0, // value length
            10, 20, 30, // value
        ];
        let record = UpdateRecord::parse(&data).unwrap();
        let expected = UpdateRecord {
            deleted: true,
            height: ChainHeight::new(200),
            value: StoreValue::from(vec![10, 20, 30]),
        };
        assert_eq!(record, expected);
    }

    #[test]
    #[should_panic]
    fn test_update_record_deserialize_malformed() {
        let data = vec![
            1, // deleted
            200, 0, 0, 0, 0, 0, 0, 0, // height
            3, 0, 0, 0, // value length
            10, 20, // value (one byte short)
        ];
        UpdateRecord::parse(&data).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_update_record_deserialize_malformed_2() {
        let data = vec![
            1, 2, 3, // too few bytes
        ];
        UpdateRecord::parse(&data).unwrap();
    }
}

#[cfg(test)]
mod journal_tests {
    use crate::declarations::basics::StoreValue;
    use crate::storage::vkv::journal::{UnitJournal, UpdateRecord};
    use crate::storage::vkv::ChainHeight;

    #[test]
    fn test_journal_serialize() {
        let journal = UnitJournal {
            api_version: 257,
            updates: vec![
                UpdateRecord {
                    deleted: false,
                    height: ChainHeight::new(10),
                    value: StoreValue::from(vec![10, 20, 30, 40, 50]),
                },
                UpdateRecord {
                    deleted: true,
                    height: ChainHeight::new(20),
                    value: StoreValue::from(vec![10, 20, 30, 40, 50, 60, 70]),
                },
                UpdateRecord {
                    deleted: true,
                    height: ChainHeight::new(21),
                    value: StoreValue::from(vec![]),
                },
            ],
        };
        let serialized = journal.serialize();
        let expected = vec![
            1, 1, 0, 0, // api version
            /* first record */
            18, 0, 0, 0, // record length
            0, // deleted
            10, 0, 0, 0, 0, 0, 0, 0, // height
            5, 0, 0, 0, // value length
            10, 20, 30, 40, 50, // value
            /* second record */
            20, 0, 0, 0, // record length
            1, // deleted
            20, 0, 0, 0, 0, 0, 0, 0, // height
            7, 0, 0, 0, // value length
            10, 20, 30, 40, 50, 60, 70, // value
            /* third record */
            13, 0, 0, 0, // record length
            1, // deleted
            21, 0, 0, 0, 0, 0, 0, 0, // height
            0, 0, 0, 0, // value length
        ];
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_journal_parse() {
        let data = vec![
            0, 2, 0, 0, // api version
            /* first record */
            13, 0, 0, 0, // record length
            1, // deleted
            21, 0, 0, 0, 0, 0, 0, 0, // height
            0, 0, 0, 0, // value length
            /* second record */
            18, 0, 0, 0, // record length
            0, // deleted
            10, 0, 0, 0, 0, 0, 0, 0, // height
            5, 0, 0, 0, // value length
            10, 20, 30, 40, 50, // value
            /* third record */
            20, 0, 0, 0, // record length
            1, // deleted
            20, 0, 0, 0, 0, 0, 0, 0, // height
            7, 0, 0, 0, // value length
            10, 20, 30, 40, 50, 60, 70, // value
        ];
        let parsed = UnitJournal::parse(&data).unwrap();
        let expected = UnitJournal {
            api_version: 512,
            updates: vec![
                UpdateRecord {
                    deleted: true,
                    height: ChainHeight::new(21),
                    value: StoreValue::from(vec![]),
                },
                UpdateRecord {
                    deleted: false,
                    height: ChainHeight::new(10),
                    value: StoreValue::from(vec![10, 20, 30, 40, 50]),
                },
                UpdateRecord {
                    deleted: true,
                    height: ChainHeight::new(20),
                    value: StoreValue::from(vec![10, 20, 30, 40, 50, 60, 70]),
                },
            ],
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    #[should_panic]
    fn test_journal_parse_malformed_record_length() {
        let data = vec![
            0, 2, 0, 0, // api version
            /* first record */
            13, 0, 0, 0, // record length
            1, // deleted
            21, 0, 0, 0, 0, 0, 0, 0, // height
            0, 0, 0, 0, // value length
            /* second record */
            20, 0, 0, 0, // record length <-- should be 18
            0, // deleted
            10, 0, 0, 0, 0, 0, 0, 0, // height
            5, 0, 0, 0, // value length
            10, 20, 30, 40, 50, // value
            /* third record */
            20, 0, 0, 0, // record length
            1, // deleted
            20, 0, 0, 0, 0, 0, 0, 0, // height
            7, 0, 0, 0, // value length
            10, 20, 30, 40, 50, 60, 70, // value
        ];
        UnitJournal::parse(&data).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_journal_parse_malformed_value_length() {
        let data = vec![
            0, 2, 0, 0, // api version
            /* first record */
            13, 0, 0, 0, // record length
            1, // deleted
            21, 0, 0, 0, 0, 0, 0, 0, // height
            0, 0, 0, 0, // value length
            /* second record */
            18, 0, 0, 0, // record length
            0, // deleted
            10, 0, 0, 0, 0, 0, 0, 0, // height
            50, 0, 0, 0, // value length <--- should be 5
            10, 20, 30, 40, 50, // value
            /* third record */
            20, 0, 0, 0, // record length
            1, // deleted
            20, 0, 0, 0, 0, 0, 0, 0, // height
            7, 0, 0, 0, // value length
            10, 20, 30, 40, 50, 60, 70, // value
        ];
        UnitJournal::parse(&data).unwrap();
    }

}
