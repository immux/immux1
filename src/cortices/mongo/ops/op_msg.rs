#![allow(dead_code)]

use std::ffi::CString;
use std::mem;

use bson::Document;
use crc::crc32;

use crate::config::VERIFY_OPMSG_CHECKSUM;
use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mongo::ops::msg_header::{serialize_msg_header, MsgHeader};
use crate::cortices::mongo::ops::opcodes::MongoOpCode::OpGetMore;
use crate::cortices::mongo::utils::{parse_bson_document, parse_cstring, parse_u32, parse_u8};
use crate::declarations::errors::UnumError::MongoParser;
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::{get_bit_u32, set_bit_u32, u32_to_u8_array, u8_array_to_u32};

const CHECK_SUM_PRESENT_DIGIT: u8 = 0;
const MORE_TO_COME_DIGIT: u8 = 1;
const EXHAUST_ALLOWED_DIGIT: u8 = 16;
const SINGLE_TYPE: u8 = 0x00;
const SEQUENCE_TYPE: u8 = 0x01;

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#flag-bits
pub struct OpMsgFlags {
    pub check_sum_present: bool,
    pub more_to_come: bool,
    pub exhaust_allowed: bool,
}

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#kind-1-document-sequence
pub struct DocumentSequence {
    section_size: u32,
    identifier: CString,
    documents: Vec<Document>,
}

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#wire-msg-sections
pub enum Section {
    Single(Document),
    Sequence(DocumentSequence),
}

#[derive(Debug)]
/// @see https://docs.mongodb.com/manual/reference/mongodb-wire-protocol/#op-msg
pub struct OpMsg {
    // standard message header
    pub message_header: MsgHeader,

    // message flags
    pub flags: OpMsgFlags,

    // data sections
    pub sections: Vec<Section>,
}

static CHECKSUM_SIZE: usize = mem::size_of::<u32>();

fn bytes_to_u32(data: &[u8]) -> UnumResult<u32> {
    if data.len() < 4 {
        Err(UnumError::MongoParser(
            MongoParserError::NotEnoughBufferSize,
        ))
    } else {
        Ok(u8_array_to_u32(&[data[0], data[1], data[2], data[3]]))
    }
}

fn separate_from_tail(buffer: &[u8], tail_len: usize) -> UnumResult<(&[u8], &[u8])> {
    if buffer.len() < tail_len {
        Err(UnumError::MongoParser(
            MongoParserError::NotEnoughBufferSize,
        ))
    } else {
        let data_len = buffer.len() - tail_len;
        Ok((&buffer[..data_len], &buffer[data_len..]))
    }
}

fn parse_checksum(buffer: &[u8]) -> UnumResult<(&[u8], &[u8])> {
    separate_from_tail(buffer, CHECKSUM_SIZE)
}

fn verify_checksum(data: &[u8], checksum: &[u8]) -> bool {
    match separate_from_tail(data, CHECKSUM_SIZE) {
        Err(_) => false,
        Ok((data, _tail)) => {
            if let Ok(checksum_u32) = bytes_to_u32(checksum) {
                crc32::checksum_castagnoli(data) == checksum_u32
            } else {
                false
            }
        }
    }
}

pub fn parse_op_msg(message_header: MsgHeader, buffer: &[u8]) -> UnumResult<OpMsg> {
    let (flag_bits_vec, next_buffer) = parse_u32(buffer)?;
    let flags = OpMsgFlags {
        check_sum_present: get_bit_u32(flag_bits_vec, CHECK_SUM_PRESENT_DIGIT),
        more_to_come: get_bit_u32(flag_bits_vec, MORE_TO_COME_DIGIT),
        exhaust_allowed: get_bit_u32(flag_bits_vec, EXHAUST_ALLOWED_DIGIT),
    };
    if flags.check_sum_present {
        let (checksum, next_buffer) = parse_checksum(&next_buffer)?;
        if VERIFY_OPMSG_CHECKSUM {
            if !verify_checksum(buffer, checksum) {
                return Err(UnumError::MongoParser(
                    MongoParserError::OpMsgChecksumMismatch,
                ));
            }
        }
    }

    let mut sections: Vec<Section> = Vec::new();
    while next_buffer.len() > 0 {
        let (kind, next_buffer) = parse_u8(next_buffer)?;
        match kind {
            SINGLE_TYPE => {
                let (doc, next_buffer) = parse_bson_document(&next_buffer)?;
                sections.push(Section::Single(doc));
            }
            SEQUENCE_TYPE => {
                let (section_size, rest_buffer) = parse_u32(&next_buffer)?;
                let (identifier, identifier_size, next_buffer) = parse_cstring(rest_buffer)?;
                let mut bson_documents_size =
                    (section_size as usize) - mem::size_of::<u32>() - identifier_size;
                let mut documents = Vec::new();
                while bson_documents_size != 0 {
                    let bson_doc_size = bytes_to_u32(&next_buffer)? as usize;
                    bson_documents_size -= bson_doc_size;
                    let (doc, next_buffer) = parse_bson_document(next_buffer)?;
                    documents.push(doc);
                }
                let document_sequence = DocumentSequence {
                    section_size,
                    identifier,
                    documents,
                };
                sections.push(Section::Sequence(document_sequence));
            }
            _ => {
                return Err(UnumError::MongoParser(MongoParserError::UnknownSectionKind));
            }
        }
    }
    return Ok(OpMsg {
        message_header,
        flags,
        sections,
    });
}

pub fn serialize_op_msg(op_msg: &OpMsg) -> UnumResult<Vec<u8>> {
    let mut res_buffer = serialize_msg_header(&op_msg.message_header);
    let mut flag_bits: u32 = 0;
    set_bit_u32(
        &mut flag_bits,
        CHECK_SUM_PRESENT_DIGIT,
        op_msg.flags.check_sum_present,
    );
    set_bit_u32(
        &mut flag_bits,
        MORE_TO_COME_DIGIT,
        op_msg.flags.more_to_come,
    );
    set_bit_u32(
        &mut flag_bits,
        EXHAUST_ALLOWED_DIGIT,
        op_msg.flags.exhaust_allowed,
    );
    res_buffer.append(&mut u32_to_u8_array(flag_bits).to_vec());

    for section in &op_msg.sections {
        let mut section_vec: Vec<u8> = Vec::new();
        match section {
            Section::Single(doc) => {
                section_vec.push(SINGLE_TYPE);
                match bson::encode_document(&mut section_vec, doc) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(UnumError::MongoSerializer(
                            MongoSerializeError::SerializeBsonError(error),
                        ));
                    }
                }
            }
            Section::Sequence(document_sequence) => {
                section_vec.push(SEQUENCE_TYPE);
                section_vec.append(&mut u32_to_u8_array(document_sequence.section_size).to_vec());
                section_vec.append(&mut document_sequence.identifier.as_bytes_with_nul().to_vec());
                for doc in &document_sequence.documents {
                    match bson::encode_document(&mut section_vec, doc) {
                        Ok(_) => {}
                        Err(error) => {
                            return Err(UnumError::MongoSerializer(
                                MongoSerializeError::SerializeBsonError(error),
                            ));
                        }
                    }
                }
            }
        }
        res_buffer.append(&mut section_vec);
    }
    return Ok(res_buffer);
}

#[cfg(test)]
mod op_msg_tests {

    use crate::cortices::mongo::ops::msg_header::parse_msg_header;
    use crate::cortices::mongo::ops::op_msg::{parse_op_msg, serialize_op_msg, Section};

    static OP_MSG_FIXTURE_SINGLE: [u8; 224] = [
        0xe0, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0xdd, 0x07, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xcb, 0x00, 0x00, 0x00, 0x08, 0x69, 0x73, 0x6d, 0x61,
        0x73, 0x74, 0x65, 0x72, 0x00, 0x01, 0x10, 0x6d, 0x61, 0x78, 0x42, 0x73, 0x6f, 0x6e, 0x4f,
        0x62, 0x6a, 0x65, 0x63, 0x74, 0x53, 0x69, 0x7a, 0x65, 0x00, 0x00, 0x00, 0x00, 0x01, 0x10,
        0x6d, 0x61, 0x78, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x53, 0x69, 0x7a, 0x65, 0x42,
        0x79, 0x74, 0x65, 0x73, 0x00, 0x00, 0x6c, 0xdc, 0x02, 0x10, 0x6d, 0x61, 0x78, 0x57, 0x72,
        0x69, 0x74, 0x65, 0x42, 0x61, 0x74, 0x63, 0x68, 0x53, 0x69, 0x7a, 0x65, 0x00, 0xa0, 0x86,
        0x01, 0x00, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x54, 0x69, 0x6d, 0x65, 0x00, 0xfe, 0x09,
        0xa6, 0xe7, 0x69, 0x01, 0x00, 0x00, 0x10, 0x6c, 0x6f, 0x67, 0x69, 0x63, 0x61, 0x6c, 0x53,
        0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x54, 0x69, 0x6d, 0x65, 0x6f, 0x75, 0x74, 0x4d, 0x69,
        0x6e, 0x75, 0x74, 0x65, 0x73, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x10, 0x6d, 0x69, 0x6e, 0x57,
        0x69, 0x72, 0x65, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x10, 0x6d, 0x61, 0x78, 0x57, 0x69, 0x72, 0x65, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e,
        0x00, 0x07, 0x00, 0x00, 0x00, 0x08, 0x72, 0x65, 0x61, 0x64, 0x4f, 0x6e, 0x6c, 0x79, 0x00,
        0x00, 0x01, 0x6f, 0x6b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, 0x00,
    ];

    #[test]
    fn test_parse_op_msg_section_single() {
        let buffer = OP_MSG_FIXTURE_SINGLE;
        let (header, next_buffer) = parse_msg_header(&buffer).unwrap();
        let op_msg = parse_op_msg(header, next_buffer).unwrap();
        assert_eq!(op_msg.flags.check_sum_present, false);
        assert_eq!(op_msg.flags.exhaust_allowed, false);
        assert_eq!(op_msg.sections.len(), 1);

        match &op_msg.sections[0] {
            Section::Single(doc) => {
                assert_eq!(doc.contains_key("ismaster"), true);
                assert_eq!(doc.contains_key("ok"), true);
                assert_eq!(doc.contains_key("readOnly"), true);
                assert_eq!(doc.contains_key("localTime"), true);
                assert_eq!(doc.contains_key("maxWriteBatchSize"), true);
                assert_eq!(doc.get_f64("ok").unwrap(), 1.0);
                assert_eq!(doc.get_bool("readOnly").unwrap(), false);
            }
            Section::Sequence(_document_sequence) => {
                assert!(
                    false,
                    "This buffer should contain only a single section type"
                );
            }
        }
    }

    #[test]
    fn test_serialize_op_msg_section_single() {
        let buffer = OP_MSG_FIXTURE_SINGLE;
        let (header, next_buffer) = parse_msg_header(&buffer).unwrap();
        let op_msg = parse_op_msg(header, next_buffer).unwrap();
        let op_msg_vec = serialize_op_msg(&op_msg).unwrap();
        assert_eq!(buffer.to_vec(), op_msg_vec);
    }

    static OP_MSG_FIXTURE_SEQUENCE: [u8; 294] = [
        0x26, 0x01, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xdd, 0x07, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xb2, 0x00, 0x00, 0x00, 0x64, 0x6f, 0x63, 0x75, 0x6d,
        0x65, 0x6e, 0x74, 0x73, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x01, 0x5f, 0x69, 0x64, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x26, 0x40, 0x02, 0x69, 0x74, 0x65, 0x6d, 0x00, 0x07, 0x00,
        0x00, 0x00, 0x70, 0x65, 0x6e, 0x63, 0x69, 0x6c, 0x00, 0x01, 0x71, 0x74, 0x79, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x49, 0x40, 0x02, 0x74, 0x79, 0x70, 0x65, 0x00, 0x05, 0x00,
        0x00, 0x00, 0x6e, 0x6f, 0x2e, 0x32, 0x00, 0x00, 0x31, 0x00, 0x00, 0x00, 0x07, 0x5f, 0x69,
        0x64, 0x00, 0x5c, 0xb1, 0x9e, 0x47, 0x0a, 0x00, 0x6f, 0x87, 0xdc, 0xd3, 0xc3, 0x80, 0x02,
        0x69, 0x74, 0x65, 0x6d, 0x00, 0x04, 0x00, 0x00, 0x00, 0x70, 0x65, 0x6e, 0x00, 0x01, 0x71,
        0x74, 0x79, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x40, 0x00, 0x34, 0x00, 0x00,
        0x00, 0x07, 0x5f, 0x69, 0x64, 0x00, 0x5c, 0xb1, 0x9e, 0x47, 0x0a, 0x00, 0x6f, 0x87, 0xdc,
        0xd3, 0xc3, 0x81, 0x02, 0x69, 0x74, 0x65, 0x6d, 0x00, 0x07, 0x00, 0x00, 0x00, 0x65, 0x72,
        0x61, 0x73, 0x65, 0x72, 0x00, 0x01, 0x71, 0x74, 0x79, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x39, 0x40, 0x00, 0x00, 0x5e, 0x00, 0x00, 0x00, 0x02, 0x69, 0x6e, 0x73, 0x65, 0x72,
        0x74, 0x00, 0x09, 0x00, 0x00, 0x00, 0x70, 0x72, 0x6f, 0x64, 0x75, 0x63, 0x74, 0x73, 0x00,
        0x08, 0x6f, 0x72, 0x64, 0x65, 0x72, 0x65, 0x64, 0x00, 0x01, 0x03, 0x6c, 0x73, 0x69, 0x64,
        0x00, 0x1e, 0x00, 0x00, 0x00, 0x05, 0x69, 0x64, 0x00, 0x10, 0x00, 0x00, 0x00, 0x04, 0xff,
        0xd9, 0xbb, 0x29, 0xec, 0xdc, 0x43, 0x1d, 0xb8, 0x6b, 0x32, 0xb4, 0x02, 0x6b, 0x63, 0x86,
        0x00, 0x02, 0x24, 0x64, 0x62, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x74, 0x65, 0x73, 0x74, 0x5f,
        0x75, 0x6e, 0x75, 0x6d, 0x5f, 0x64, 0x62, 0x00, 0x00,
    ];

    #[test]
    fn test_parse_op_msg_section_sequence() {
        let buffer = OP_MSG_FIXTURE_SEQUENCE;
        let (header, next_buffer) = parse_msg_header(&buffer).unwrap();
        let op_msg = parse_op_msg(header, next_buffer).unwrap();
        assert_eq!(op_msg.flags.check_sum_present, false);
        assert_eq!(op_msg.flags.exhaust_allowed, false);
        assert_eq!(op_msg.sections.len(), 2);
        match &op_msg.sections[0] {
            Section::Single(_doc) => {
                assert!(false, "The first section should be sequence type.");
            }
            Section::Sequence(document_sequence) => {
                assert_eq!(document_sequence.documents.len(), 3);
                assert_eq!(document_sequence.documents[0].contains_key("_id"), true);
                assert_eq!(document_sequence.documents[0].get_f64("_id").unwrap(), 11.0);
                assert_eq!(document_sequence.documents[1].contains_key("item"), true);
                assert_eq!(document_sequence.documents[1].contains_key("item"), true);
                assert_eq!(
                    document_sequence.documents[1].get_str("item").unwrap(),
                    "pen"
                );
            }
        }
        match &op_msg.sections[1] {
            Section::Single(doc) => {
                assert_eq!(doc.contains_key("insert"), true);
                assert_eq!(doc.get_str("insert").unwrap(), "products");
            }
            Section::Sequence(_document_sequence) => {
                assert!(false, "The second section should be single type.");
            }
        }
    }

    #[test]
    fn test_serialize_op_msg_section_sequence() {
        let buffer = OP_MSG_FIXTURE_SEQUENCE;
        let (header, next_buffer) = parse_msg_header(&buffer).unwrap();
        let op_msg = parse_op_msg(header, next_buffer).unwrap();
        let op_msg_vec = serialize_op_msg(&op_msg).unwrap();
        assert_eq!(buffer.to_vec(), op_msg_vec);
    }
}
