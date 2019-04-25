use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::{u8_array_to_u32, u8_array_to_u64};
use std::ffi::CString;
use std::mem::size_of;

pub fn parse_u8(buffer: &[u8], error: UnumError) -> UnumResult<(u8, usize)> {
    let field_size = size_of::<u8>();
    if buffer.len() < field_size {
        return Err(error);
    } else {
        Ok((buffer[0], field_size))
    }
}

pub fn parse_u32(buffer: &[u8], error: UnumError) -> UnumResult<(u32, usize)> {
    let field_size = size_of::<u32>();
    if buffer.len() < field_size {
        return Err(error);
    } else {
        Ok((
            u8_array_to_u32(&[buffer[0], buffer[1], buffer[2], buffer[3]]),
            field_size,
        ))
    }
}

pub fn parse_u64(buffer: &[u8], error: UnumError) -> UnumResult<(u64, usize)> {
    let field_size = size_of::<u64>();
    if buffer.len() < field_size {
        return Err(error);
    } else {
        Ok((
            u8_array_to_u64(&[
                buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                buffer[7],
            ]),
            field_size,
        ))
    }
}

pub fn parse_cstring(buffer: &[u8], error: UnumError) -> UnumResult<(String, usize)> {
    match buffer.iter().position(|&r| r == b'\0') {
        None => {
            return Err(error);
        }
        Some(terminal_index) => {
            let new_value = if terminal_index == 0 {
                CString::new("")
            } else {
                CString::new(&buffer[..terminal_index])
            };
            match new_value {
                Err(_nulerror) => {
                    return Err(error);
                }
                Ok(value) => match value.to_str() {
                    Ok(val) => {
                        return Ok((val.to_string(), terminal_index + 1));
                    }
                    Err(_) => {
                        return Err(error);
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod cortices_utils_tests {
    use crate::cortices::mongo::error::MongoParserError;
    use crate::cortices::mongo::ops::msg_header::parse_msg_header;
    use crate::cortices::utils::{parse_cstring, parse_u32, parse_u64, parse_u8};
    use crate::declarations::errors::UnumError;

    #[test]
    fn test_parse_u8() {
        let buffer = [0x11];
        let res = parse_u8(
            &buffer,
            UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
        );
        let (num, index_off_set) = res.unwrap();
        assert_eq!(17, num);
        assert_eq!(index_off_set, 1);
    }

    #[test]
    #[should_panic]
    fn test_parse_u8_error() {
        let buffer = [];
        parse_u8(
            &buffer,
            UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
        )
        .unwrap();
    }

    #[test]
    fn test_parse_u64() {
        let res = parse_u64(
            &[0x0d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
        );
        let (num, index_off_set) = res.unwrap();
        assert_eq!(269, num);
        assert_eq!(index_off_set, 8);
    }

    #[test]
    #[should_panic]
    fn test_parse_u64_error() {
        parse_u64(
            &[0x0d],
            UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
        )
        .unwrap();
    }

    #[test]
    fn test_parse_u32() {
        let res = parse_u32(
            &[0x0d, 0x01, 0x00, 0x00],
            UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
        );
        let (num, index_off_set) = res.unwrap();
        assert_eq!(269, num);
        assert_eq!(index_off_set, 4);
    }

    #[test]
    #[should_panic]
    fn test_parse_u32_error() {
        parse_u32(
            &[0x0d],
            UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
        )
        .unwrap();
    }

    static OP_QUERY_FIXTURE: [u8; 269] = [
        0x0d, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd4, 0x07, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x61, 0x64, 0x6d, 0x69, 0x6e, 0x2e, 0x24, 0x63, 0x6d, 0x64,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xe6, 0x00, 0x00, 0x00, 0x10, 0x69,
        0x73, 0x4d, 0x61, 0x73, 0x74, 0x65, 0x72, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x63, 0x6c,
        0x69, 0x65, 0x6e, 0x74, 0x00, 0xcb, 0x00, 0x00, 0x00, 0x03, 0x61, 0x70, 0x70, 0x6c, 0x69,
        0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x00, 0x1d, 0x00, 0x00, 0x00, 0x02, 0x6e, 0x61, 0x6d,
        0x65, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x4d, 0x6f, 0x6e, 0x67, 0x6f, 0x44, 0x42, 0x20, 0x53,
        0x68, 0x65, 0x6c, 0x6c, 0x00, 0x00, 0x03, 0x64, 0x72, 0x69, 0x76, 0x65, 0x72, 0x00, 0x3a,
        0x00, 0x00, 0x00, 0x02, 0x6e, 0x61, 0x6d, 0x65, 0x00, 0x18, 0x00, 0x00, 0x00, 0x4d, 0x6f,
        0x6e, 0x67, 0x6f, 0x44, 0x42, 0x20, 0x49, 0x6e, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x6c, 0x20,
        0x43, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x00, 0x02, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e,
        0x00, 0x06, 0x00, 0x00, 0x00, 0x34, 0x2e, 0x30, 0x2e, 0x31, 0x00, 0x00, 0x03, 0x6f, 0x73,
        0x00, 0x56, 0x00, 0x00, 0x00, 0x02, 0x74, 0x79, 0x70, 0x65, 0x00, 0x07, 0x00, 0x00, 0x00,
        0x44, 0x61, 0x72, 0x77, 0x69, 0x6e, 0x00, 0x02, 0x6e, 0x61, 0x6d, 0x65, 0x00, 0x09, 0x00,
        0x00, 0x00, 0x4d, 0x61, 0x63, 0x20, 0x4f, 0x53, 0x20, 0x58, 0x00, 0x02, 0x61, 0x72, 0x63,
        0x68, 0x69, 0x74, 0x65, 0x63, 0x74, 0x75, 0x72, 0x65, 0x00, 0x07, 0x00, 0x00, 0x00, 0x78,
        0x38, 0x36, 0x5f, 0x36, 0x34, 0x00, 0x02, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00,
        0x07, 0x00, 0x00, 0x00, 0x31, 0x38, 0x2e, 0x32, 0x2e, 0x30, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_parse_cstring() {
        let buffer = OP_QUERY_FIXTURE;
        let mut init_index: usize = 0;
        let (_, index_offset) = parse_msg_header(&buffer[init_index..]).unwrap();
        init_index += index_offset;
        let (_, index_offset) = parse_u32(
            &buffer[init_index..],
            UnumError::MongoParser(MongoParserError::NotEnoughBufferSize),
        )
        .unwrap();
        init_index += index_offset;
        let (res, _) = parse_cstring(
            &buffer[init_index..],
            UnumError::MongoParser(MongoParserError::ParseStringError),
        )
        .unwrap();
        assert_eq!(res, "admin.$cmd");
    }

    #[test]
    #[should_panic]
    fn test_parse_cstring_error() {
        let buffer = [0x70, 0x70, 0x6c, 0x69];
        parse_cstring(
            &buffer,
            UnumError::MongoParser(MongoParserError::ParseStringError),
        )
        .unwrap();
    }
}
