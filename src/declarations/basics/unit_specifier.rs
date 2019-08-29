use serde::{Deserialize, Serialize};

use crate::declarations::basics::{GroupingLabel, UnitId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnitSpecifier(GroupingLabel, UnitId);

impl UnitSpecifier {
    pub fn new(grouping: GroupingLabel, id: UnitId) -> Self {
        Self(grouping, id)
    }
    pub fn into_components(self) -> (GroupingLabel, UnitId) {
        return (self.0, self.1);
    }
    pub fn get_grouping(&self) -> &GroupingLabel {
        &self.0
    }
    pub fn get_id(&self) -> UnitId {
        self.1
    }
}
