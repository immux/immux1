use crate::cortices::mysql::error::MySQLParserError;
use crate::cortices::utils::parse_u8;
use crate::declarations::errors::{ImmuxError, ImmuxResult};

/// @see https://dev.mysql.com/doc/internals/en/character-set.html#packet-Protocol::CharacterSet
// TODO: character set contains too many options, here we just shows a few common character sets issue #60.
pub const LATIN1_SWEDISH_CI: u8 = 8;
pub const UTF8_GENERAL_CI: u8 = 33;
// TODO: Couldn't find character set 45, issue #76.
pub const UTF8MB4_GENERAL_CI: u8 = 45;
pub const BINARY: u8 = 63;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum CharacterSet {
    Latin1SwedishCi = LATIN1_SWEDISH_CI,
    Utf8GeneralCi = UTF8_GENERAL_CI,
    Binary = BINARY,
    Utf8mb4GeneralCi = UTF8MB4_GENERAL_CI,
}

pub fn get_character_set_value(character_set: CharacterSet) -> u8 {
    match character_set {
        CharacterSet::Latin1SwedishCi => LATIN1_SWEDISH_CI,
        CharacterSet::Utf8GeneralCi => UTF8_GENERAL_CI,
        CharacterSet::Binary => BINARY,
        CharacterSet::Utf8mb4GeneralCi => UTF8MB4_GENERAL_CI,
    }
}

pub fn pick_character_set(character_set_value: u8) -> ImmuxResult<CharacterSet> {
    match character_set_value {
        LATIN1_SWEDISH_CI => Ok(CharacterSet::Latin1SwedishCi),
        UTF8_GENERAL_CI => Ok(CharacterSet::Utf8GeneralCi),
        BINARY => Ok(CharacterSet::Binary),
        UTF8MB4_GENERAL_CI => Ok(CharacterSet::Utf8mb4GeneralCi),
        _ => Err(ImmuxError::MySQLParser(
            MySQLParserError::UnknownCharacterSetValue(character_set_value),
        )),
    }
}

pub fn parse_character_set(buffer: &[u8]) -> ImmuxResult<(CharacterSet, usize)> {
    let (character_set_value, offset) = parse_u8(&buffer)?;
    let character_set = pick_character_set(character_set_value)?;
    Ok((character_set, offset))
}

#[cfg(test)]
mod character_set_tests {
    use crate::cortices::mysql::character_set::{
        get_character_set_value, pick_character_set, CharacterSet, BINARY, LATIN1_SWEDISH_CI,
        UTF8_GENERAL_CI,
    };

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

    #[test]
    fn test_pick_character_set() -> Result<(), String> {
        let character_set_value = 8;
        match pick_character_set(character_set_value).unwrap() {
            CharacterSet::Latin1SwedishCi => {}
            _ => {
                return Err(String::from(
                    "This should be CharacterSet::Latin1SwedishCi!",
                ));
            }
        }

        let character_set_value = 33;
        match pick_character_set(character_set_value).unwrap() {
            CharacterSet::Utf8GeneralCi => {}
            _ => {
                return Err(String::from("This should be CharacterSet::Utf8GeneralCi!"));
            }
        }

        let character_set_value = 45;
        match pick_character_set(character_set_value).unwrap() {
            CharacterSet::Utf8mb4GeneralCi => {}
            _ => {
                return Err(String::from(
                    "This should be CharacterSet::Utf8mb4GeneralCi!",
                ));
            }
        }

        let character_set_value = 63;
        match pick_character_set(character_set_value).unwrap() {
            CharacterSet::Binary => {}
            _ => {
                return Err(String::from("This should be CharacterSet::Binary!"));
            }
        }

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_pick_character_set_error() {
        let character_set_value = 99;
        pick_character_set(character_set_value).unwrap();
    }
}
