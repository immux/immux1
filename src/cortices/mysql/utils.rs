use crate::cortices::mysql::error::{MySQLParserError, MySQLSerializeError};
use crate::cortices::utils::{parse_u16, parse_u64, parse_u8, DeserializationError};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::{u16_to_u8_array, u32_to_u8_array, u64_to_u8_array, u8_array_to_u32};
use std::mem::size_of;
use std::num::ParseIntError;
use std::str;

pub const MYSQL_PACKET_HEADER_LENGTH: usize = 4;

pub fn parse_length_encoded_integer(buffer: &[u8]) -> UnumResult<(usize, usize)> {
    let mut index: usize = 0;
    let (identifier, offset) = parse_u8(&buffer[index..])?;
    index += offset;
    if identifier < 0xfb {
        return Ok((identifier as usize, index));
    } else if identifier == 0xfc {
        let (value, offset) = parse_u16(&buffer[index..])?;
        return Ok((value as usize, index + offset));
    } else if identifier == 0xfd {
        let (value, offset) = parse_u32_with_length_3(&buffer[index..])?;
        return Ok((value as usize, index + offset));
    } else if identifier == 0xfe {
        let (value, offset) = parse_u64(&buffer[index..])?;
        return Ok((value as usize, index + offset));
    } else {
        return Err(UnumError::MySQLParser(MySQLParserError::UnknownIdentifier(
            identifier,
        )));
    }
}

pub fn serialize_length_encoded_integer(num: u128) -> UnumResult<Vec<u8>> {
    if num < 251 {
        return Ok(vec![num as u8]);
    } else if num < 2u128.pow(16) {
        let mut res = vec![0xfc];
        let mut val = u16_to_u8_array(num as u16).to_vec();
        res.append(&mut val);
        return Ok(res);
    } else if num < 2u128.pow(24) {
        let mut res = vec![0xfd];
        let mut val = u32_to_u8_array(num as u32).to_vec();
        res.append(&mut val);
        return Ok(res);
    } else if num < 2u128.pow(64) {
        let mut res = vec![0xfe];
        let mut val = u64_to_u8_array(num as u64).to_vec();
        res.append(&mut val);
        return Ok(res);
    } else {
        return Err(UnumError::MySQLSerializer(
            MySQLSerializeError::LengthEncodedIntegerTooLarge,
        ));
    }
}

pub fn serialize_length_encoded_string(string: String) -> UnumResult<Vec<u8>> {
    let mut res = Vec::new();
    let mut string_length_vec = serialize_length_encoded_integer(string.len() as u128)?;
    res.append(&mut string_length_vec);
    let mut string_vec = string.as_bytes().to_vec();
    res.append(&mut string_vec);
    return Ok(res);
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
        Err(UnumError::Deserialization(
            DeserializationError::InsufficientDataWidthU32,
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

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    let mut res = Vec::new();
    for i in (0..s.len()).step_by(2) {
        let mut hex_str = if i == s.len() - 1 {
            &s[i..i + 1]
        } else {
            &s[i..i + 2]
        };
        res.push(u8::from_str_radix(hex_str, 16)?);
    }
    return Ok(res);
}

// TODO: We are using this method to judge the current state of connection phase, see issue 87.
pub enum ConnectionStatePhase {
    LoginRequest = 1,
    AuthSwitchResponse = 3,
}
pub fn get_packet_number(buffer: &[u8]) -> UnumResult<ConnectionStatePhase> {
    let packet_header_size = 4;
    if buffer.len() < packet_header_size {
        return Err(DeserializationError::InsufficientDataWidthU32.into());
    }
    match &buffer[3] {
        1 => Ok(ConnectionStatePhase::LoginRequest),
        3 => Ok(ConnectionStatePhase::AuthSwitchResponse),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod mysql_utils_tests {

    use crate::cortices::mysql::utils::{
        decode_hex, parse_length_encoded_integer, parse_string_with_fixed_length, parse_u16,
        parse_u32_with_length_3, serialize_length_encoded_integer, serialize_length_encoded_string,
        u32_to_u8_array_with_length_3,
    };

    #[test]
    fn test_parse_u16() {
        let buffer: &[u8] = &[0x01, 0x02];
        let (res, _) = parse_u16(buffer).unwrap();
        assert_eq!(res, 513);
    }

    #[test]
    #[should_panic]
    fn test_parse_u16_error() {
        let buffer: &[u8] = &[0x01];
        parse_u16(buffer).unwrap();
    }

    #[test]
    fn test_parse_length_encoded_integer() {
        let buffer = [0x01];
        let (val, offset) = parse_length_encoded_integer(&buffer).unwrap();
        assert_eq!(val, 1);
        assert_eq!(offset, 1);

        let buffer = [0xfc, 0xfb, 0x00];
        let (val, offset) = parse_length_encoded_integer(&buffer).unwrap();
        assert_eq!(val, 251);
        assert_eq!(offset, 3);

        let buffer = [0xfc, 0x22, 0x01];
        let (val, offset) = parse_length_encoded_integer(&buffer).unwrap();
        assert_eq!(val, 290);
        assert_eq!(offset, 3);

        let buffer = [0xfd, 0xea, 0x75, 0xd1];
        let (val, offset) = parse_length_encoded_integer(&buffer).unwrap();
        assert_eq!(val, 13727210);
        assert_eq!(offset, 4);

        let buffer = [0xfe, 0x91, 0x5c, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
        let (val, offset) = parse_length_encoded_integer(&buffer).unwrap();
        assert_eq!(val, 16800913);
        assert_eq!(offset, 9);
    }

    #[test]
    fn test_serialize_length_encoded_integer() {
        let num = 250;
        let res = serialize_length_encoded_integer(num as u128).unwrap();
        assert_eq!(res, [0xfa].to_vec());

        let num = 2u32.pow(16) - 1;
        let res = serialize_length_encoded_integer(num as u128).unwrap();
        assert_eq!(res, [0xfc, 0xff, 0xff].to_vec());

        let num = 2u32.pow(24) - 1;
        let res = serialize_length_encoded_integer(num as u128).unwrap();
        assert_eq!(res, [0xfd, 0xff, 0xff, 0xff, 0x00].to_vec());

        let num = 2u128.pow(64) - 1;
        let res = serialize_length_encoded_integer(num).unwrap();
        assert_eq!(
            res,
            [0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff].to_vec()
        );
    }

    #[test]
    #[should_panic]
    fn test_serialize_length_encoded_integer_error() {
        let num = 2u128.pow(64);
        serialize_length_encoded_integer(num as u128).unwrap();
    }

    #[test]
    fn test_serialize_length_encoded_string() {
        let string = "test".to_string();
        let res = serialize_length_encoded_string(string).unwrap();
        assert_eq!(res, [0x04, 0x74, 0x65, 0x73, 0x74].to_vec());
    }

    #[test]
    fn test_parse_string_with_fixed_length() {
        let string = "test".to_string();
        let buffer = string.as_bytes().to_vec();
        let (res, _) = parse_string_with_fixed_length(&buffer, buffer.len()).unwrap();
        assert_eq!(res, string);
    }

    #[test]
    #[should_panic]
    fn test_parse_string_with_fixed_length_error() {
        let string = "test".to_string();
        let buffer = string.as_bytes().to_vec();
        parse_string_with_fixed_length(&buffer, buffer.len() + 1).unwrap();
    }

    #[test]
    fn test_parse_u32_with_length_3() {
        let buffer = [0x01, 0x00, 0x00];
        let (res, offset) = parse_u32_with_length_3(&buffer).unwrap();
        assert_eq!(res, 1);
        assert_eq!(offset, 3);
    }

    #[test]
    #[should_panic]
    fn test_parse_u32_with_length_3_error() {
        let buffer = [0x01];
        parse_u32_with_length_3(&buffer).unwrap();
    }

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
    fn test_decode_hex() {
        let hex_str = "01";
        let res = decode_hex(hex_str).unwrap();
        assert_eq!(res, [0x01].to_vec());

        let hex_str = "fb";
        let res = decode_hex(hex_str).unwrap();
        assert_eq!(res, [0xfb].to_vec());

        let hex_str = "fb3c";
        let res = decode_hex(hex_str).unwrap();
        assert_eq!(res, [0xfb, 0x3c].to_vec());

        let hex_str = "1fb";
        let res = decode_hex(hex_str).unwrap();
        assert_eq!(res, [0x1f, 0x0b].to_vec());

        let hex_str = "10ff";
        let res = decode_hex(hex_str).unwrap();
        assert_eq!(res, [0x10, 0xff].to_vec());
    }

    #[test]
    #[should_panic]
    fn test_decode_hex_error() {
        let hex_str = "hello world";
        let res = decode_hex(hex_str).unwrap();
    }
}
