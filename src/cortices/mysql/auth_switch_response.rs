use crate::cortices::mysql::utils::parse_u32_with_length_3;
use crate::cortices::utils::{parse_cstring, parse_u8};
use crate::declarations::errors::ImmuxResult;

pub struct AuthSwitchResponse {
    pub payload_length: u32,
    pub packet_number: u8,
    pub data: Option<String>,
}

pub fn parse_auth_switch_response(buffer: &[u8]) -> ImmuxResult<AuthSwitchResponse> {
    let mut index: usize = 0;
    let (payload_length, offset) = parse_u32_with_length_3(&buffer[index..])?;
    index += offset;
    let (packet_number, offset) = parse_u8(&buffer[index..])?;
    index += offset;

    let mut data = None;
    if index != buffer.len() {
        let (data_val, _offset) = parse_cstring(&buffer[index..])?;
        data = Some(data_val);
    }

    let auth_switch_response = AuthSwitchResponse {
        payload_length,
        packet_number,
        data,
    };

    return Ok(auth_switch_response);
}

#[cfg(test)]
mod auth_switch_response_tests {

    use crate::cortices::mysql::auth_switch_response::{
        parse_auth_switch_response,
    };

    #[test]
    fn test_serialize_auth_switch_response() {
        let buffer = [0x00, 0x00, 0x00, 0x03];
        let auth_swtich_response = parse_auth_switch_response(&buffer).unwrap();
        assert_eq!(auth_swtich_response.payload_length, 0);
        assert_eq!(auth_swtich_response.packet_number, 3);
        assert_eq!(auth_swtich_response.data, None);
    }
}
