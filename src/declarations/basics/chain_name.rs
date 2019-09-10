use serde::{Deserialize, Serialize};

use crate::storage::instructions::StoreNamespace;
use crate::utils::utf8_to_string;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ChainName(Vec<u8>);

impl ChainName {
    pub fn new(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<&str> for ChainName {
    fn from(data: &str) -> ChainName {
        ChainName(data.as_bytes().to_owned())
    }
}

impl ToString for ChainName {
    fn to_string(&self) -> String {
        utf8_to_string(&self.0)
    }
}

impl From<StoreNamespace> for ChainName {
    fn from(ns: StoreNamespace) -> Self {
        ChainName(ns.as_bytes().to_vec())
    }
}

impl From<ChainName> for StoreNamespace {
    fn from(name: ChainName) -> Self {
        StoreNamespace::new(name.as_bytes())
    }
}

#[cfg(test)]
mod test_chain_names {
    use crate::declarations::basics::ChainName;

    #[test]
    fn check_reversibility() {
        let name_str = "chain";
        let name = ChainName::from(name_str);
        assert_eq!(name.to_string(), name_str)
    }
}
