use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupingLabel(Vec<u8>);

impl GroupingLabel {
    pub fn new(data: &[u8]) -> Self {
        GroupingLabel(data.to_vec())
    }
    pub fn marshal(&self) -> Vec<u8> {
        let data = &self.0;
        let mut result: Vec<u8> = Vec::new();
        result.push(data.len() as u8);
        result.extend_from_slice(data);
        result
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for GroupingLabel {
    fn from(data: Vec<u8>) -> Self {
        return GroupingLabel::new(&data);
    }
}

impl From<&[u8]> for GroupingLabel {
    fn from(data: &[u8]) -> Self {
        return GroupingLabel::new(data);
    }
}

impl Into<Vec<u8>> for GroupingLabel {
    fn into(self) -> Vec<u8> {
        return self.0;
    }
}

impl From<&GroupingLabel> for Vec<u8> {
    fn from(data: &GroupingLabel) -> Vec<u8> {
        data.as_bytes().to_vec()
    }
}

#[cfg(test)]
mod grouping_label_tests {
    use crate::declarations::basics::GroupingLabel;

    #[test]
    fn test_serialize() {
        let label_str = "hello";
        let label = GroupingLabel::from(label_str.as_bytes());
        let bytes = label.marshal();
        let expected = vec![
            5, // length,
            104, 101, 108, 108, 111, //"hello"
        ];
        assert_eq!(bytes, expected);
    }
}
