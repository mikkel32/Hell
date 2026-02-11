/// Decodes an XOR-encoded byte array with the given key
pub fn decode(encoded: &[u8], key: u8) -> String {
    let decoded_bytes: Vec<u8> = encoded.iter().map(|&b| b ^ key).collect();
    String::from_utf8(decoded_bytes).unwrap_or_else(|_| "ERROR".to_string())
}
