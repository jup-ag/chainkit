pub fn to_base64(data: impl AsRef<[u8]>) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(data)
}

pub fn from_base64(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.decode(value)
}

pub fn from_base64_to_array_32(label: &str, s: &str) -> [u8; 32] {
    use base64::{engine::general_purpose, Engine as _};
    let v = general_purpose::STANDARD
        .decode(s)
        .unwrap_or_else(|e| panic!("{label}: invalid base64 ({e})"));
    assert!(
        v.len() == 32,
        "{label}: expected 32 bytes, got {} (did you pass the *shared* key?)",
        v.len()
    );
    let mut out = [0u8; 32];
    out.copy_from_slice(&v);
    out
}

pub fn parse_string_as_byte_array(input: &str) -> Option<Vec<u8>> {
    if input.starts_with('[') && input.ends_with(']') {
        let slice = input.get(1..(input.len() - 1))?;
        Some(
            slice
                .split(',')
                .filter_map(|e| e.trim().parse::<u8>().ok())
                .collect::<Vec<u8>>(),
        )
    } else {
        None
    }
}
