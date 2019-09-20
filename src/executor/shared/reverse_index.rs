use std::collections::hash_map::IntoIter as HashMapIntoIter;
use std::collections::HashMap;

use serde_json::Value as JsonValue;

use crate::declarations::basics::{IdList, PropertyName, PropertyNameList, UnitContent, UnitId};
use crate::declarations::errors::ImmuxResult;

#[derive(Debug)]
pub enum ReverseIndexError {
    CannotParseJson,
    UnexpectedNumberType,
    UnimplementedIndexingPropertyType,
}

// `Property` would have been `UnitContent`, if `f64` were `Eq`.
// Unfortunately, as `f64` is not `Eq` (because of NaN), and `UnitContent` can hold f64,
// `UnitContent` is not hashable and cannot be key.
type Name = PropertyName;
type Property = Vec<u8>;

/// {
///    age: {
///       1: [id1, id2],
///       2: [id3, id4],
///       3: [id5, id9],
///       4: [id10, id8],
///    },
///    first_name: {
///       "julie": [id5, id8],
///       "mike": [id10, id9],
///       "tom": [id1, id2],
///    },
/// }
///
#[derive(Debug)]
pub struct ReverseIndex {
    inner: HashMap<(Name, Property), IdList>,
}

impl IntoIterator for ReverseIndex {
    type Item = ((Name, Property), IdList);
    type IntoIter = HashMapIntoIter<(Name, Property), IdList>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl ReverseIndex {
    /// Creates an empty reverse index
    pub fn new() -> Self {
        let inner = HashMap::new();
        return ReverseIndex { inner };
    }

    /// Register an id matching "name: property".
    /// For example, for JSON(id=1) {age: 80}, `index.add_to_index(age, 80, 1)` would be called.
    pub fn add_to_index(&mut self, name: &PropertyName, property_bytes: &[u8], id: UnitId) -> () {
        let key = (name.to_owned(), property_bytes.to_owned());
        match self.inner.get(&key) {
            Some(id_list) => {
                let mut new_list = id_list.to_owned();
                new_list.push(id);
                self.inner.insert(key, new_list)
            }
            None => {
                let new_list = IdList::new(vec![id]);
                self.inner.insert(key, new_list)
            }
        };
    }

    /// A more convenient version of `add_to_index` for JSONs
    pub fn index_new_json(
        &mut self,
        id: UnitId,
        json: &JsonValue,
        target_name: &PropertyName,
    ) -> ImmuxResult<()> {
        let name_str = target_name.to_string();
        match json.get(name_str) {
            // property doesn't exist on the json
            None => return Ok(()),
            // Property does exist (but could be null)
            Some(json_property) => {
                let property: UnitContent = match &json_property {
                    JsonValue::String(string) => UnitContent::String(string.clone()),
                    JsonValue::Bool(boolean) => UnitContent::Bool(*boolean),
                    JsonValue::Number(number) => {
                        let data = if let Some(num) = number.as_f64() {
                            num
                        } else {
                            return Err(ReverseIndexError::UnexpectedNumberType.into());
                        };
                        UnitContent::Float64(data)
                    }
                    JsonValue::Null => UnitContent::Nil,
                    _ => {
                        return Err(ReverseIndexError::UnimplementedIndexingPropertyType.into());
                    }
                };
                self.add_to_index(target_name, &property.marshal(), id);
                Ok(())
            }
        }
    }

    pub fn get(&self, name: &PropertyName, property: &UnitContent) -> IdList {
        let key = (name.to_owned(), property.marshal());
        match self.inner.get(&key) {
            None => IdList::new(vec![]),
            Some(list) => list.to_owned(),
        }
    }

    /// Add multiple JSONs with multiple names to index; basically a stronger `index_new_json`.
    pub fn from_jsons(
        data: &[(UnitId, String)],
        names_to_index: &PropertyNameList,
    ) -> ImmuxResult<Self> {
        let mut index = Self::new();
        for row in data {
            let (id, json_string) = row;
            match serde_json::from_str::<JsonValue>(json_string) {
                Err(_error) => return Err(ReverseIndexError::CannotParseJson.into()),
                Ok(json_value) => {
                    for name in names_to_index.clone() {
                        index.index_new_json(*id, &json_value, &name)?;
                    }
                }
            }
        }
        Ok(index)
    }
}

#[cfg(test)]
mod reverse_index_test {
    use serde_json::Value as JsonValue;

    use crate::declarations::basics::{PropertyName, PropertyNameList, UnitContent, UnitId};
    use crate::executor::shared::ReverseIndex;
    use crate::utils::utf8_to_string;

    #[test]
    fn test_single_name_single_property_indexing() {
        let mut index = ReverseIndex::new();
        let name = PropertyName::new("name".as_bytes());
        let property = UnitContent::String(String::from("property"));
        let count: u128 = 100;

        // Insert & index
        for i in 0..count {
            let id = UnitId::new(i as u128);
            index.add_to_index(&name, &property.marshal(), id);
        }

        // Check
        let id_list = index.get(&name, &property);
        assert_eq!(id_list.as_slice().len() as u128, count);
        for i in 0..count {
            assert!(id_list.as_slice().contains(&UnitId::new(i)))
        }
    }

    #[test]
    fn test_indexing_various_names() {
        let raw_data: Vec<(&str, &str, u128)> = vec![
            ("company", "A", 1),
            ("company", "B", 2),
            ("planet", "Mars", 3),
            ("planet", "Earth", 4),
            ("company", "B", 5),
            ("company", "A", 6),
            ("planet", "Pluto", 7),
            ("planet", "Earth", 8),
        ];

        let data: Vec<(PropertyName, UnitContent, UnitId)> = raw_data
            .into_iter()
            .map(|(p, c, i)| {
                (
                    PropertyName::new(p.as_bytes()),
                    UnitContent::String(c.to_string()),
                    UnitId::new(i),
                )
            })
            .collect();

        let mut index = ReverseIndex::new();

        for datum in &data {
            index.add_to_index(&datum.0, &datum.1.marshal(), datum.2);
        }

        for datum in &data {
            let ids = index.get(&datum.0, &datum.1);
            assert!(ids.as_slice().contains(&datum.2))
        }
    }

    type Row = (u128, f64, &'static str, bool);

    const NUMBER_NAME: &str = "num";
    const STRING_NAME: &str = "string";
    const BOOLEAN_NAME: &str = "bool";

    fn row_to_json_str(row: &Row) -> String {
        format!(
            r#"{{"{}": {}, "{}": "{}", "{}": {}}}"#,
            NUMBER_NAME, row.1, STRING_NAME, row.2, BOOLEAN_NAME, row.3
        )
    }

    fn row_to_json(row: &Row) -> JsonValue {
        let json_str = row_to_json_str(&row);
        serde_json::from_str::<JsonValue>(&json_str).unwrap()
    }

    fn get_standard_data_table() -> Vec<Row> {
        vec![
            (1, 1.0, "hello", true),
            (2, 2.0, "world", true),
            (3, 1.0, "hello", false),
            (4, 0.8, "hello", true),
            (5, 0.0, "hello", false),
            (6, 0.0, "hello", false),
            (7, 0.0, "hello", false),
            (8, 0.0, "hello", false),
            (9, 1.0, "hello", false),
        ]
    }

    #[test]
    fn test_index_new_json() {
        let table = get_standard_data_table();
        assert!(table.len() > 0);

        let name_to_index = PropertyName::from(NUMBER_NAME);

        // Build table
        let mut index = ReverseIndex::new();
        for row in &table {
            let json_value = row_to_json(row);
            index
                .index_new_json(UnitId::new(row.0), &json_value, &name_to_index)
                .unwrap();
        }

        // Check
        for row in &table {
            let property = UnitContent::Float64(row.1);
            let id_list = index.get(&name_to_index, &property);
            assert!(id_list.as_slice().contains(&UnitId::new(row.0)));
            assert_eq!(
                id_list.as_slice().len(),
                table
                    .iter()
                    .filter(|original| original.1 == row.1)
                    .collect::<Vec<_>>()
                    .len()
            );
        }
    }

    #[test]
    fn test_index_jsons() {
        let table = get_standard_data_table();
        assert!(table.len() > 0);

        let json_table: Vec<(UnitId, String)> = table
            .iter()
            .map(|row| (UnitId::new(row.0), row_to_json_str(row)))
            .collect();
        let names_to_index = PropertyNameList::new(vec![
            PropertyName::new(NUMBER_NAME.as_bytes()),
            PropertyName::new(STRING_NAME.as_bytes()),
            PropertyName::new(BOOLEAN_NAME.as_bytes()),
        ]);

        // Build table
        let index = ReverseIndex::from_jsons(&json_table, &names_to_index).unwrap();

        // Check
        for row in &table {
            for name in names_to_index.clone() {
                match utf8_to_string(name.as_bytes()).as_ref() {
                    NUMBER_NAME => {
                        let id_list = index.get(&name, &UnitContent::Float64(row.1));
                        assert!(id_list.as_slice().contains(&UnitId::new(row.0)));
                        assert_eq!(
                            id_list.as_slice().len(),
                            table
                                .iter()
                                .filter(|original| original.1 == row.1)
                                .collect::<Vec<_>>()
                                .len()
                        );
                    }
                    STRING_NAME => {
                        let id_list = index.get(&name, &UnitContent::String(row.2.to_string()));
                        assert!(id_list.as_slice().contains(&UnitId::new(row.0)));
                        assert_eq!(
                            id_list.as_slice().len(),
                            table
                                .iter()
                                .filter(|original| original.2 == row.2)
                                .collect::<Vec<_>>()
                                .len()
                        );
                    }
                    BOOLEAN_NAME => {
                        let id_list = index.get(&name, &UnitContent::Bool(row.3));
                        assert!(id_list.as_slice().contains(&UnitId::new(row.0)));
                        assert_eq!(
                            id_list.as_slice().len(),
                            table
                                .iter()
                                .filter(|original| original.3 == row.3)
                                .collect::<Vec<_>>()
                                .len()
                        );
                    }
                    _ => panic!("Unexpected name {:?}", name),
                };
            }
        }
    }
}
