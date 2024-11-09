pub fn format_u8_hex(data: &[u8]) -> String {
    let mut string = String::new();
    for (i, e) in data.iter().enumerate() {
        if i % 8 == 0 {
            string.push('\n');
        }
        string.push_str(&format!("{:02x} ", e))
    }
    string.push('\n');

    string
}
