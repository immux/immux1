use crate::declarations::errors::UnumResult;
use crate::storage::core::UnumCore;
use crate::utils::pretty_dump;

pub fn mysql_cortex_process_incoming_message(
    bytes: &[u8],
    _core: &mut UnumCore,
) -> UnumResult<Option<Vec<u8>>> {
    pretty_dump(bytes);
    unimplemented!()
}

pub fn mysql_cortex_process_first_connection(_core: &mut UnumCore) -> UnumResult<Option<Vec<u8>>> {
    unimplemented!()
}
