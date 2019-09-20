use crate::config::KVKeySigil;
use crate::declarations::basics::property_names::PropertyName;
use crate::declarations::basics::{GroupingLabel, StoreKey, UnitContent};

pub fn get_store_key_of_indexed_id_list(
    grouping: &GroupingLabel,
    name: &PropertyName,
    property: &UnitContent,
) -> StoreKey {
    let mut key_bytes: Vec<u8> = Vec::new();
    key_bytes.push(KVKeySigil::ReverseIndexIdList as u8);
    key_bytes.extend(grouping.marshal());
    key_bytes.extend(name.marshal());
    key_bytes.extend(property.marshal());
    return StoreKey::new(&key_bytes);
}

#[cfg(test)]
mod indexed_id_list_key_tests {
    use crate::declarations::basics::{GroupingLabel, PropertyName, UnitContent};
    use crate::executor::shared::get_store_key_of_indexed_id_list;

    #[test]
    fn test_key_bytes() {
        let grouping = GroupingLabel::new(&[0xff, 0x00]);
        let name = PropertyName::new(&[0x01, 0x02, 0x03]);
        let content = UnitContent::Bytes(vec![0xaa, 0xbb, 0xcc, 0xdd]);
        let key = get_store_key_of_indexed_id_list(&grouping, &name, &content);
        let expected = [
            0xa0, // sigil
            0x02, 0xff, 0x00, // grouping
            0x03, 0x01, 0x02, 0x03, // name
            0xff, 0x04, 0xaa, 0xbb, 0xcc, 0xdd, // content
        ];
        assert_eq!(key.as_slice(), &expected)
    }
}
