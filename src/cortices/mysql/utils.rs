use crate::cortices::mysql::error::MySQLSerializeError;
use crate::cortices::mysql::initial_handshake_packet::{
    CharacterSet, BINARY, LATIN1_SWEDISH_CI, UTF8_GENERAL_CI,
};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::u32_to_u8_array;

pub fn u32_to_u8_array_with_length_3(x: u32) -> UnumResult<[u8; 3]> {
    if x > 2u32.pow(24) - 1 {
        return Err(UnumError::MySQLSerializer(
            MySQLSerializeError::PacketSizeTooLarge,
        ));
    } else {
        let mut res: [u8; 3] = Default::default();
        res.copy_from_slice(&u32_to_u8_array(x)[0..3]);
        return Ok(res);
    }
}

pub fn get_character_set_value(character_set: CharacterSet) -> u8 {
    match character_set {
        CharacterSet::Latin1SwedishCi => LATIN1_SWEDISH_CI,
        CharacterSet::Utf8GeneralCi => UTF8_GENERAL_CI,
        CharacterSet::Binary => BINARY,
    }
}

#[cfg(test)]
mod mysql_utils_tests {

    use crate::cortices::mysql::initial_handshake_packet::{
        CharacterSet, BINARY, LATIN1_SWEDISH_CI, UTF8_GENERAL_CI,
    };
    use crate::cortices::mysql::utils::{get_character_set_value, u32_to_u8_array_with_length_3};

    #[test]
    fn test_u32_to_u8_array_with_length_3() {
        let number: u32 = 74;
        let res = u32_to_u8_array_with_length_3(number).unwrap();
        assert_eq!(res[0], 0x4a);
        assert_eq!(res[1], 0x00);
        assert_eq!(res[2], 0x00);
    }

    #[test]
    #[should_panic]
    fn test_u32_to_u8_array_with_length_3_error() {
        let number: u32 = 2u32.pow(24);
        u32_to_u8_array_with_length_3(number).unwrap();
    }

    #[test]
    fn test_get_character_set_value() {
        assert_eq!(get_character_set_value(CharacterSet::Binary), BINARY);
        assert_eq!(
            get_character_set_value(CharacterSet::Utf8GeneralCi),
            UTF8_GENERAL_CI
        );
        assert_eq!(
            get_character_set_value(CharacterSet::Latin1SwedishCi),
            LATIN1_SWEDISH_CI
        );
    }

}
