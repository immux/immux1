use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct TransactionId(u64);

impl TransactionId {
    pub fn new(data: u64) -> Self {
        TransactionId(data)
    }
    pub fn increment(&mut self) -> () {
        self.0 += 1
    }
    pub fn as_int(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod transaction_id_tests {
    use crate::storage::tkv::transaction_id::TransactionId;

    #[test]
    fn test_increment() {
        let mut id = TransactionId::new(0);
        id.increment();
        assert_eq!(id.as_int(), 1);
    }
}
