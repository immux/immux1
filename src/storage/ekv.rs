/*
 *  Encrypted key-value store
**/

use crate::storage::kv::KeyValueStore;

trait EncryptedKeyValueStore: KeyValueStore {}
