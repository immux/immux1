use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::utils::parse_u16;
use crate::declarations::errors::{ImmuxError, ImmuxResult};
use crate::declarations::instructions::{
    Answer, AtomicGetOneInstruction, AtomicSetInstruction, GetTargetSpec, Instruction,
    SetTargetSpec,
};
use crate::storage::core::{CoreStore, ImmuxDBCore};
use crate::utils::{get_bit_u16, set_bit_u16};

/// @see https://dev.mysql.com/doc/internals/en/status-flags.html#packet-Protocol::StatusFlags

#[derive(Copy, Clone)]
pub struct ServerStatusFlags {
    pub intrans: bool,
    pub autocommit: bool,
    pub more_results_exists: bool,
    pub no_good_index_used: bool,
    pub no_index_used: bool,
    pub cursor_exists: bool,
    pub last_row_sent: bool,
    pub db_dropped: bool,
    pub no_backslash_escapes: bool,
    pub metadata_changed: bool,
    pub query_was_slow: bool,
    pub ps_out_params: bool,
    pub intrans_readonly: bool,
    pub session_state_changed: bool,
}

pub fn serialize_status_flags(flags_struct: &ServerStatusFlags) -> u16 {
    let mut result: u16 = 0;
    set_bit_u16(&mut result, 0, flags_struct.intrans);
    set_bit_u16(&mut result, 1, flags_struct.autocommit);
    set_bit_u16(&mut result, 2, flags_struct.more_results_exists);
    set_bit_u16(&mut result, 3, flags_struct.no_good_index_used);
    set_bit_u16(&mut result, 4, flags_struct.no_index_used);
    set_bit_u16(&mut result, 5, flags_struct.cursor_exists);
    set_bit_u16(&mut result, 6, flags_struct.last_row_sent);
    set_bit_u16(&mut result, 7, flags_struct.db_dropped);
    set_bit_u16(&mut result, 8, flags_struct.no_backslash_escapes);
    set_bit_u16(&mut result, 9, flags_struct.metadata_changed);
    set_bit_u16(&mut result, 10, flags_struct.query_was_slow);
    set_bit_u16(&mut result, 11, flags_struct.ps_out_params);
    set_bit_u16(&mut result, 12, flags_struct.intrans_readonly);
    set_bit_u16(&mut result, 13, flags_struct.session_state_changed);

    return result;
}

pub fn parse_status_flags(flags_vec: u16) -> ServerStatusFlags {
    let intrans = get_bit_u16(flags_vec, 0);
    let autocommit = get_bit_u16(flags_vec, 1);
    let more_results_exists = get_bit_u16(flags_vec, 2);
    let no_good_index_used = get_bit_u16(flags_vec, 3);
    let no_index_used = get_bit_u16(flags_vec, 4);
    let cursor_exists = get_bit_u16(flags_vec, 5);
    let last_row_sent = get_bit_u16(flags_vec, 6);
    let db_dropped = get_bit_u16(flags_vec, 7);
    let no_backslash_escapes = get_bit_u16(flags_vec, 8);
    let metadata_changed = get_bit_u16(flags_vec, 9);
    let query_was_slow = get_bit_u16(flags_vec, 10);
    let ps_out_params = get_bit_u16(flags_vec, 11);
    let intrans_readonly = get_bit_u16(flags_vec, 12);
    let session_state_changed = get_bit_u16(flags_vec, 13);

    ServerStatusFlags {
        intrans,
        autocommit,
        more_results_exists,
        no_good_index_used,
        no_index_used,
        cursor_exists,
        last_row_sent,
        db_dropped,
        no_backslash_escapes,
        metadata_changed,
        query_was_slow,
        ps_out_params,
        intrans_readonly,
        session_state_changed,
    }
}

const SERVER_STATUS_FLAGS_KEY: &str = "_SERVER_STATUS_FLAGS";

pub fn save_server_status_flags(buffer: &[u8], core: &mut ImmuxDBCore) -> ImmuxResult<()> {
    let instruction = AtomicSetInstruction {
        targets: vec![SetTargetSpec {
            key: SERVER_STATUS_FLAGS_KEY.as_bytes().to_vec(),
            value: buffer.to_vec(),
        }],
        increment_height: false,
    };

    match core.execute(&Instruction::AtomicSet(instruction)) {
        Err(_error) => Err(ImmuxError::MySQLParser(
            MySQLParserError::CannotSetServerStatusFlags,
        )),
        Ok(_) => Ok(()),
    }
}

pub fn load_server_status_flags(core: &mut ImmuxDBCore) -> ImmuxResult<ServerStatusFlags> {
    let instruction = AtomicGetOneInstruction {
        target: GetTargetSpec {
            key: SERVER_STATUS_FLAGS_KEY.as_bytes().to_vec(),
            height: None,
        },
    };
    match core.execute(&Instruction::AtomicGetOne(instruction)) {
        Err(_error) => {
            return Err(ImmuxError::MySQLSerializer(
                MySQLSerializeError::CannotReadServerStatusFlags,
            ));
        }
        Ok(answer) => match answer {
            Answer::GetOneOk(get_answer) => {
                let target = &get_answer.item;
                let (status_flags, _) = parse_u16(target)?;
                let res = parse_status_flags(status_flags);
                return Ok(res);
            }
            _ => {
                return Err(ImmuxError::MySQLSerializer(
                    MySQLSerializeError::CannotReadServerStatusFlags,
                ));
            }
        },
    }
}

#[cfg(test)]
mod server_status_flags_tests {

    use crate::config::{DEFAULT_CHAIN_NAME, DEFAULT_PERMANENCE_PATH};
    use crate::cortices::mysql::server_status_flags::{
        load_server_status_flags, parse_status_flags, save_server_status_flags,
        serialize_status_flags, ServerStatusFlags,
    };
    use crate::storage::core::ImmuxDBCore;
    use crate::storage::kv::KeyValueEngine;

    #[test]
    fn test_serialize_status_flags() {
        let status_flags = ServerStatusFlags {
            intrans: false,
            autocommit: true,
            more_results_exists: false,
            no_good_index_used: false,
            no_index_used: false,
            cursor_exists: false,
            last_row_sent: false,
            db_dropped: false,
            no_backslash_escapes: false,
            metadata_changed: false,
            query_was_slow: false,
            ps_out_params: false,
            intrans_readonly: false,
            session_state_changed: false,
        };
        assert_eq!(serialize_status_flags(&status_flags), 0x0002);
    }

    #[test]
    fn test_parse_status_flags() {
        let buffer = 0x0002;
        let res = parse_status_flags(buffer);
        assert_eq!(serialize_status_flags(&res), 0x0002);
    }

    #[test]
    fn test_save_load_server_status_flags() {
        let engine_choice = KeyValueEngine::HashMap;
        let server_status_flags_buffer = [0x02, 0x00];
        let mut core = ImmuxDBCore::new(
            &engine_choice,
            DEFAULT_PERMANENCE_PATH,
            DEFAULT_CHAIN_NAME.as_bytes(),
        )
        .unwrap();
        save_server_status_flags(&server_status_flags_buffer, &mut core).unwrap();
        let res = load_server_status_flags(&mut core).unwrap();
        assert_eq!(res.intrans, false);
        assert_eq!(res.autocommit, true);
        assert_eq!(res.more_results_exists, false);
        assert_eq!(res.no_good_index_used, false);
        assert_eq!(res.no_index_used, false);
        assert_eq!(res.cursor_exists, false);
        assert_eq!(res.last_row_sent, false);
        assert_eq!(res.db_dropped, false);
        assert_eq!(res.no_backslash_escapes, false);
        assert_eq!(res.metadata_changed, false);
        assert_eq!(res.query_was_slow, false);
        assert_eq!(res.ps_out_params, false);
        assert_eq!(res.intrans_readonly, false);
        assert_eq!(res.session_state_changed, false);
    }
}
