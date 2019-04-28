#![allow(dead_code)]

use std::ffi::CString;

use bson::Document;

use crate::cortices::mongo::error::{MongoParserError, MongoSerializeError};
use crate::cortices::mongo::ops::msg_header::{serialize_msg_header, MsgHeader};
use crate::cortices::mongo::utils::{parse_bson_document, parse_cstring, parse_u32, parse_u8};
use crate::declarations::errors::{UnumError, UnumResult};
use crate::utils::{get_bit_u32, set_bit_u32, u32_to_u8_array, u8_array_to_u32};
use std::mem;

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

    // optional CRC-32C checksum
    pub checksum: Option<u32>,
}

pub fn parse_op_msg(message_header: MsgHeader, buffer: &[u8]) -> UnumResult<OpMsg> {
    let mut index: usize = 0;
    let (flags_bits_vec, offset) = parse_u32(&buffer)?;
    index += offset;
    let check_sum_present = get_bit_u32(flags_bits_vec, CHECK_SUM_PRESENT_DIGIT);
    let more_to_come = get_bit_u32(flags_bits_vec, MORE_TO_COME_DIGIT);
    let exhaust_allowed = get_bit_u32(flags_bits_vec, EXHAUST_ALLOWED_DIGIT);
    let flags = OpMsgFlags {
        check_sum_present,
        more_to_come,
        exhaust_allowed,
    };
    let mut sections: Vec<Section> = Vec::new();

    loop {
        if (check_sum_present && index + mem::size_of::<u32>() == buffer.len())
            || (!check_sum_present && index == buffer.len())
        {
            break;
        }
        let (kind, offset) = parse_u8(&buffer[index..])?;
        index += offset;
        match kind {
            SINGLE_TYPE => {
                let (doc, offset) = parse_bson_document(&buffer[index..])?;
                index += offset;
                sections.push(Section::Single(doc));
            }
            SEQUENCE_TYPE => {
                let (section_size, offset) = parse_u32(&buffer[index..])?;
                index += offset;
                let (identifier, offset) = parse_cstring(&buffer[index..])?;
                index += offset;
                let mut bson_documents_size = (section_size as usize)
                    - mem::size_of::<u32>()
                    - identifier.as_bytes_with_nul().len();
                let mut documents = Vec::new();
                while bson_documents_size != 0 {
                    let bson_doc_size = u8_array_to_u32(&[
                        buffer[index],
                        buffer[index + 1],
                        buffer[index + 2],
                        buffer[index + 3],
                    ]) as usize;
                    bson_documents_size -= bson_doc_size;
                    let (doc, offset) = parse_bson_document(&buffer[index..])?;
                    index += offset;
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
                return Err(UnumError::MongoParser(MongoParserError::UnkownSectionKind));
            }
        }
    }
    let checksum = if check_sum_present {
        Some(parse_u32(&buffer[index..])?.0)
    } else {
        None
    };
    let op_msg = OpMsg {
        message_header,
        flags,
        sections,
        checksum,
    };
    return Ok(op_msg);
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
        let mut index: usize = 0;
        let buffer = OP_MSG_FIXTURE_SINGLE;
        let (header, offset) = parse_msg_header(&buffer[index..]).unwrap();
        index += offset;
        let op_msg = parse_op_msg(header, &buffer[index..]).unwrap();
        assert_eq!(op_msg.flags.check_sum_present, false);
        assert_eq!(op_msg.flags.exhaust_allowed, false);
        assert_eq!(op_msg.sections.len(), 1);

        match &op_msg.sections[0] {
            Section::Single(doc) => {
                assert!(doc.contains_key("ismaster"));
                assert!(doc.contains_key("ok"));
                assert!(doc.contains_key("readOnly"));
                assert!(doc.contains_key("localTime"));
                assert!(doc.contains_key("maxWriteBatchSize"));
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
        let mut index: usize = 0;
        let buffer = OP_MSG_FIXTURE_SINGLE;
        let (header, offset) = parse_msg_header(&buffer[index..]).unwrap();
        index += offset;
        let op_msg = parse_op_msg(header, &buffer[index..]).unwrap();
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
    fn test_parse_op_msg_section_sequence() -> Result<(), String> {
        let buffer = OP_MSG_FIXTURE_SEQUENCE;
        let mut index: usize = 0;
        let (header, offset) = parse_msg_header(&buffer[index..]).unwrap();
        index += offset;
        let op_msg = parse_op_msg(header, &buffer[index..]).unwrap();
        assert_eq!(op_msg.flags.check_sum_present, false);
        assert_eq!(op_msg.flags.exhaust_allowed, false);
        assert_eq!(op_msg.sections.len(), 2);
        match &op_msg.sections[0] {
            Section::Single(_doc) => {
                return Err(String::from("The first section should be sequence type."));
            }
            Section::Sequence(document_sequence) => {
                assert_eq!(document_sequence.documents.len(), 3);
                assert!(document_sequence.documents[0].contains_key("_id"));
                assert_eq!(document_sequence.documents[0].get_f64("_id").unwrap(), 11.0);
                assert!(document_sequence.documents[0].contains_key("item"));
                assert!(document_sequence.documents[1].contains_key("item"));
                assert_eq!(
                    document_sequence.documents[1].get_str("item").unwrap(),
                    "pen"
                );
            }
        }
        match &op_msg.sections[1] {
            Section::Single(doc) => {
                assert!(doc.contains_key("insert"));
                assert_eq!(doc.get_str("insert").unwrap(), "products");
                Ok(())
            }
            Section::Sequence(_document_sequence) => {
                Err(String::from("The second section should be single type."))
            }
        }
    }

    #[test]
    fn test_serialize_op_msg_section_sequence() {
        let buffer = OP_MSG_FIXTURE_SEQUENCE;
        let mut index: usize = 0;
        let (header, offset) = parse_msg_header(&buffer[index..]).unwrap();
        index += offset;
        let op_msg = parse_op_msg(header, &buffer[index..]).unwrap();
        let op_msg_vec = serialize_op_msg(&op_msg).unwrap();
        assert_eq!(buffer.to_vec(), op_msg_vec);
    }
}
