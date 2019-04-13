pub fn utf8_to_string(bytes: &[u8]) -> String {
    let result = std::str::from_utf8(bytes);
    match result {
        Err(_error) => String::from(""),
        Ok(s) => String::from(s),
    }
}

pub fn u64_to_u8_array(x: u64) -> [u8; 8] {
    let b7 = ((x >> 56) & 0xff) as u8;
    let b6 = ((x >> 48) & 0xff) as u8;
    let b5 = ((x >> 40) & 0xff) as u8;
    let b4 = ((x >> 32) & 0xff) as u8;
    let b3 = ((x >> 24) & 0xff) as u8;
    let b2 = ((x >> 16) & 0xff) as u8;
    let b1 = ((x >> 8) & 0xff) as u8;
    let b0 = ((x >> 0) & 0xff) as u8;

    [b0, b1, b2, b3, b4, b5, b6, b7]
}

pub fn u8_array_to_u64(data: &[u8; 8]) -> u64 {
    (((data[0] as u64) << 0)
        + ((data[1] as u64) << 8)
        + ((data[2] as u64) << 16)
        + ((data[3] as u64) << 24)
        + ((data[4] as u64) << 32)
        + ((data[5] as u64) << 40)
        + ((data[6] as u64) << 48)
        + ((data[7] as u64) << 56))
        .into()
}

pub fn u8_array_to_u32(data: &[u8; 4]) -> u32 {
    (((data[0] as u32) << 0)
        + ((data[1] as u32) << 8)
        + ((data[2] as u32) << 16)
        + ((data[3] as u32) << 24))
        .into()
}

pub fn get_bit_u32(input: u32, digit: u8) -> bool {
    if digit < 32 {
        input & (1u32 << digit) != 0
    } else {
        false
    }
}

pub fn set_bit_u32(int: &mut u32, digit: u8, value: bool) {
    if value {
        *int |= 1u32 << digit;
    } else {
        *int &= !(1u32 << digit);
    }
}

/**
Print a byte array like this:
0000 | 0d 01 00 00 00 00 00 00 00 00 00 00 d4 07 00 00
0010 | 00 00 00 00 61 64 6d 69 6e 2e 24 63 6d 64 00 00
0020 | 00 00 00 01 00 00 00 e6 00 00 00 10 69 73 4d 61
**/
pub fn pretty_dump(buffer: &[u8]) -> () {
    let mut i = 0;
    while i < buffer.len() {
        if i % 16 == 0 {
            if i > 0 {
                print!("\n");
            }
            print!("{:04x} | ", i);
        }
        print!("{:02x} ", buffer[i]);
        i += 1;
    }
    print!("\n");
}

pub fn u32_to_u8_array(x: u32) -> [u8; 4] {
    let b3 = ((x >> 24) & 0xff) as u8;
    let b2 = ((x >> 16) & 0xff) as u8;
    let b1 = ((x >> 8) & 0xff) as u8;
    let b0 = ((x >> 0) & 0xff) as u8;

    [b0, b1, b2, b3]
}
