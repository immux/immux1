#![allow(dead_code)]

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
