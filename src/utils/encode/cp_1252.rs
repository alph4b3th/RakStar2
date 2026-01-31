use encoding_rs::WINDOWS_1252;

pub fn to_cp1252(s: &str) -> Vec<u8> {
    let (encoded, _, _) = WINDOWS_1252.encode(s);
    encoded.into_owned()
}

pub fn cp1252_bytes_to_str(bytes: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

