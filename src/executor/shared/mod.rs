mod indexed_names_list;

use crate::config::KVKeySigil;
use crate::declarations::basics::property_names::PropertyName;
use crate::declarations::basics::{GroupingLabel, StoreKey, UnitContent};
use crate::utils::u32_to_u8_array;
pub use indexed_names_list::{
    get_indexed_names_list, get_indexed_names_list_with_fallback, set_indexed_names_list,
};

pub fn get_store_key_of_indexed_id_list(
    grouping: &GroupingLabel,
    name: &PropertyName,
    property: &UnitContent,
) -> StoreKey {
    let mut key_bytes: Vec<u8> = Vec::new();

    let grouping_bytes: Vec<u8> = grouping.to_owned().into();
    let name_bytes: Vec<u8> = name.to_owned().into();
    let property_bytes = property.marshal();

    key_bytes.push(KVKeySigil::ReverseIndexIdList as u8);

    key_bytes.push(grouping_bytes.len() as u8);
    key_bytes.extend(grouping_bytes);

    key_bytes.push(name_bytes.len() as u8);
    key_bytes.extend(name_bytes);

    key_bytes.extend_from_slice(&u32_to_u8_array(property_bytes.len() as u32));
    key_bytes.extend_from_slice(&property_bytes);

    return StoreKey::new(&key_bytes);
}
