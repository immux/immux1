use crate::cortices::mysql::error::MySQLSerializeError;
use crate::cortices::mysql::utils::{
    decode_hex, u32_to_u8_array_with_length_3, MYSQL_PACKET_HEADER_LENGTH,
};
use crate::declarations::errors::{ImmuxError, ImmuxResult};

pub struct AuthSwitchRequest {
    pub payload_length: u32,
    pub packet_number: u8,
    pub status: u8,
    pub plugin_name: String,
    pub plugin_data: String,
}

pub fn serialize_auth_switch_request(
    auth_switch_request: AuthSwitchRequest,
) -> ImmuxResult<Vec<u8>> {
    let mut res = Vec::new();
    res.append(&mut u32_to_u8_array_with_length_3(auth_switch_request.payload_length)?.to_vec());
    res.push(auth_switch_request.packet_number);
    res.push(auth_switch_request.status);
    let mut plugin_name_vec = auth_switch_request.plugin_name.clone().into_bytes();
    plugin_name_vec.push(0x00);
    res.append(&mut plugin_name_vec);
    match decode_hex(&auth_switch_request.plugin_data) {
        Err(error) => {
            return Err(ImmuxError::MySQLSerializer(
                MySQLSerializeError::SerializePluginDataError(error),
            ));
        }
        Ok(mut data) => {
            res.append(&mut data);
            if res.len() - MYSQL_PACKET_HEADER_LENGTH != auth_switch_request.payload_length as usize
            {
                return Err(ImmuxError::MySQLSerializer(
                    MySQLSerializeError::SerializeAuthPluginDataError,
                ));
            }
            return Ok(res);
        }
    }
}

#[cfg(test)]
mod auth_switch_request_tests {

    use crate::cortices::mysql::auth_switch_request::{
        serialize_auth_switch_request, AuthSwitchRequest,
    };

    #[test]
    fn test_serialize_auth_switch_request() {
        let payload_length = 44;
        let packet_number = 2;
        let status = 0xfe;
        let plugin_name = "mysql_native_password".to_string();
        let plugin_data = "4640602c36261e662d4848106437014f404e2a0300".to_string();
        let auth_switch_request = AuthSwitchRequest {
            payload_length,
            packet_number,
            status,
            plugin_name,
            plugin_data,
        };
        let res = serialize_auth_switch_request(auth_switch_request).unwrap();
        let buffer = [
            0x2c, 0x00, 0x00, 0x02, 0xfe, 0x6d, 0x79, 0x73, 0x71, 0x6c, 0x5f, 0x6e, 0x61, 0x74,
            0x69, 0x76, 0x65, 0x5f, 0x70, 0x61, 0x73, 0x73, 0x77, 0x6f, 0x72, 0x64, 0x00, 0x46,
            0x40, 0x60, 0x2c, 0x36, 0x26, 0x1e, 0x66, 0x2d, 0x48, 0x48, 0x10, 0x64, 0x37, 0x01,
            0x4f, 0x40, 0x4e, 0x2a, 0x03, 0x00,
        ];
        assert_eq!(res, buffer.to_vec());
    }
}
