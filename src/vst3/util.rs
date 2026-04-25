pub fn extract_cstring(bytes: &[i8]) -> String {
	let len = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
	let u8_bytes: Vec<u8> = bytes[..len].iter().map(|&b| b as u8).collect();
	String::from_utf8_lossy(&u8_bytes).to_string()
}

pub fn extract_cstring_utf16(bytes: &[u16]) -> String {
	let len = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
	let u16_str: Vec<u16> = bytes[..len].to_vec();
	String::from_utf16_lossy(&u16_str).to_string()
}
