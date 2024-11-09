use std::collections::HashMap;

const CODING_TABLE: [(char, u8); 36] = [
    ('а', 0x01), ('б', 0x02), ('в', 0x03), ('г', 0x04),
    ('д', 0x05), ('е', 0x06), ('ё', 0x07), ('ж', 0x08),
    ('з', 0x09), ('и', 0x10), ('й', 0x11), ('к', 0x12),
    ('л', 0x13), ('м', 0x14), ('н', 0x15), ('о', 0x16),
    ('п', 0x17), ('р', 0x18), ('с', 0x19), ('т', 0x20),
    ('у', 0x21), ('ф', 0x22), ('х', 0x23), ('ц', 0x24),
    ('ч', 0x25), ('ш', 0x26), ('щ', 0x27), ('ъ', 0x28),
    ('ы', 0x29), ('ь', 0x30), ('э', 0x31), ('ю', 0x32),
    ('я', 0x33), (' ', 0x34), ('.', 0x35), (',', 0x36),
];

pub fn encode(text: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(text.len());
    let enc_map = HashMap::from(CODING_TABLE);

    for i in text.chars() {
        result.push(enc_map[&i]);
    }

    result
}

pub fn decode(text: &Vec<u8>) -> String {
    let mut result = String::with_capacity(text.len());
    let dec_map: HashMap<u8, char> = CODING_TABLE.iter().map(|&(k, v)| (v, k)).collect();

    for i in text {
        result.push(dec_map[i]);
    }

    result
}
