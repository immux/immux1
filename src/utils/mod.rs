mod bools;
mod debug;
mod floats;
mod ints;
mod strings;
mod varint;

pub use bools::{bool_to_u8, u8_to_bool};
pub use debug::pretty_dump;
pub use floats::{f64_to_u8_array, u8_array_to_f64};
pub use ints::{
    get_bit_u16, get_bit_u32, set_bit_u16, set_bit_u32, u128_to_u8_array, u16_to_u8_array,
    u32_to_u8_array, u64_to_u8_array, u8_array_to_u128, u8_array_to_u16, u8_array_to_u32,
    u8_array_to_u64,
};
pub use strings::utf8_to_string;
pub use varint::{varint_decode, varint_encode, VarIntError};
