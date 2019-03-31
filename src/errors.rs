pub enum UnumError {
    InitializationFail,
    ReadError,
    WriteError,

    SerializationFail,
    DeserializationFail,

    UrlParseError,
}

pub fn explain_error(error: UnumError) -> &'static str {
    match error {
        UnumError::InitializationFail => "initialization failed",
        UnumError::ReadError => "read error",
        UnumError::WriteError => "write error",
        UnumError::DeserializationFail => "deserialization failed",
        UnumError::SerializationFail => "serialization failed",
        UnumError::UrlParseError => "url parse error",
    }
}
