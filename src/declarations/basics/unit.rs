use serde::{Deserialize, Serialize};

use crate::declarations::basics::{UnitContent, UnitId};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Unit {
    pub id: UnitId,
    pub content: UnitContent,
}

impl ToString for Unit {
    fn to_string(&self) -> String {
        format!("{}|{}", self.id.as_int(), self.content.to_string())
    }
}
