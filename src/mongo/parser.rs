use crate::mongo::header_parser::parse_msg_header;
use crate::mongo::op_query_parser::parse_op_query;

pub fn parse_mongo_wire_protocol_buffer(buffer: &[u8], bytes_read: usize) {
    println!("Total {} bytes were read", bytes_read);
    if let Ok((header, buffer)) = parse_msg_header(buffer) {
        if let Ok(op_query) = parse_op_query(header, buffer) {
            println!("{:#?}", op_query.query);
        }
    }
}
