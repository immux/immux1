use serde::{Deserialize, Serialize};

use crate::config::MAX_GROUPING_LABEL_LENGTH;
use crate::utils::utf8_to_string;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupingLabel(Vec<u8>);

impl GroupingLabel {
    pub fn new(data: &[u8]) -> Self {
        if data.len() < MAX_GROUPING_LABEL_LENGTH {
            GroupingLabel(data.to_vec())
        } else {
            GroupingLabel(data[0..MAX_GROUPING_LABEL_LENGTH].to_vec())
        }
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

impl From<&str> for GroupingLabel {
    fn from(data: &str) -> GroupingLabel {
        GroupingLabel(data.as_bytes().to_owned())
    }
}

impl From<&GroupingLabel> for Vec<u8> {
    fn from(data: &GroupingLabel) -> Vec<u8> {
        data.as_bytes().to_vec()
    }
}

impl ToString for GroupingLabel {
    fn to_string(&self) -> String {
        utf8_to_string(self.as_bytes())
    }
}

#[cfg(test)]
mod grouping_label_tests {
    use crate::declarations::basics::GroupingLabel;

    #[test]
    fn test_grouping_label_overflow() {
        let data = [0; 1000];
        let label = GroupingLabel::new(&data);
        assert_eq!(label.as_bytes().len(), 128)
    }

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

    #[test]
    fn test_to_string() {
        let expected_label_str = "hello";
        let label = GroupingLabel::from(expected_label_str.as_bytes());
        let actual_label_str = label.to_string();
        assert_eq!(expected_label_str, actual_label_str);
    }
}
