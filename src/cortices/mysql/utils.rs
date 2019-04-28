use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::utils::{parse_u64, parse_u8};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::{u32_to_u8_array, u8_array_to_u16, u8_array_to_u32};
use std::mem::size_of;
use std::str;

pub fn parse_u16(buffer: &[u8]) -> UnumResult<(u16, usize)> {
    let field_size = size_of::<u16>();
    if buffer.len() < field_size {
        Err(UnumError::MySQLParser(
            MySQLParserError::NotEnoughBufferSize,
        ))
    } else {
        Ok((u8_array_to_u16(&[buffer[0], buffer[1]]), field_size))
    }
}

pub fn parse_length_encoded_integer(buffer: &[u8]) -> UnumResult<(usize, usize)> {
    let mut init_index: usize = 0;
    let (identifier, index_offset) = parse_u8(
        &buffer[init_index..],
        UnumError::MySQLParser(MySQLParserError::NotEnoughBufferSize),
    )?;
    init_index += index_offset;
    if identifier < 0xfb {
        return Ok((identifier as usize, init_index));
    } else if identifier == 0xfc {
        let (value, index_offset) = parse_u16(&buffer[init_index..])?;
        return Ok((value as usize, init_index + index_offset));
    } else if identifier == 0xfd {
        let (value, index_offset) = parse_u32_with_length_3(&buffer[init_index..])?;
        return Ok((value as usize, init_index + index_offset));
    } else if identifier == 0xfe {
        let (value, index_offset) = parse_u64(
            &buffer[init_index..],
            UnumError::MySQLParser(MySQLParserError::NotEnoughBufferSize),
        )?;
        return Ok((value as usize, init_index + index_offset));
    } else {
        return Err(UnumError::MySQLParser(MySQLParserError::UnknownIdentifier(
            identifier,
        )));
    }
}

pub fn parse_string_with_fixed_length(buffer: &[u8], length: usize) -> UnumResult<(String, usize)> {
    match str::from_utf8(&buffer[0..length]) {
        Ok(val) => {
            return Ok((val.to_string(), length));
        }
        Err(_) => {
            return Err(UnumError::MySQLParser(MySQLParserError::ParseStringError));
        }
    }
}

pub fn parse_u32_with_length_3(buffer: &[u8]) -> UnumResult<(u32, usize)> {
    let field_size = 3;
    if buffer.len() < field_size {
        Err(UnumError::MySQLParser(
            MySQLParserError::NotEnoughBufferSize,
        ))
    } else {
        let res = u8_array_to_u32(&[buffer[0], buffer[1], buffer[2], 0x00]);
        Ok((res, field_size))
    }
}

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

#[cfg(test)]
mod mysql_utils_tests {

    use crate::cortices::mysql::utils::u32_to_u8_array_with_length_3;

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

}
