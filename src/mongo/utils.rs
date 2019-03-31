use crate::mongo::constants::*;
use crate::mongo::error::ParseError;
use std::ffi::CString;

extern crate bson;

pub fn parse_cstring(buffer: &[u8]) -> Result<(CString, &[u8]), ParseError> {
    match buffer.iter().position(|&r| r == b'\0') {
        None => Err(ParseError::NoZeroTrailingInCstringBuffer),
        Some(cstring_size) => {
            let cstring_from_bytes = |buffer: &[u8]| CString::new(buffer);
            let (cstring, buffer) = parse_field(
                buffer,
                cstring_size,
                ParseError::CstringContainZeroByte,
                &cstring_from_bytes,
            )?;
            // cstring is a little bit special, we need to skip the trailing zero, which is one byte, when we slice the buffer
            Ok((cstring, &buffer[1..]))
        }
    }
}

pub fn parse_bson_document(buffer: &[u8]) -> Result<(bson::Document, &[u8]), ParseError> {
    let (bson_size, next_buffer) = parse_u32(buffer)?;
    match bson::decode_document(&mut &(*buffer)[0..(bson_size as usize)]) {
        Err(error) => Err(ParseError::ParseBsonError(error)),
        Ok(bson_document) => Ok((bson_document, &buffer[(bson_size as usize)..])),
    }
}

pub fn parse_field<'a, T, E>(
    buffer: &'a [u8],
    field_size: usize,
    error: ParseError,
    extract_fn: &Fn(&[u8]) -> Result<T, E>,
) -> Result<(T, &'a [u8]), ParseError> {
    if field_size > buffer.len() {
        eprintln!("Buffer doesn't have enough size for slicing");
        return Err(ParseError::NotEnoughBufferSize);
    }
    let next_buffer = &buffer[field_size..];
    match extract_fn(&buffer[0..field_size]) {
        Ok(val) => Ok((val, next_buffer)),
        Err(_) => Err(error),
    }
}

pub fn parse_u32(buffer: &[u8]) -> Result<(u32, &[u8]), ParseError> {
    let read_u32: fn(&[u8]) -> Result<u32, ()> = |bytes: &[u8]| {
        Ok((bytes[0] as u32)
            + (bytes[1] as u32 * 256)
            + (bytes[2] as u32 * 65536)
            + (bytes[3] as u32 * 16777216))
    };
    parse_field(
        buffer,
        U32_BYTE_SIZE,
        ParseError::NotEnoughBufferSize,
        &read_u32,
    )
}
