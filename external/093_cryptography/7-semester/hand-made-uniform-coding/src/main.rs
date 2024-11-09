use std::fs::File;

use hand_made_uniform_coding::{encode, decode};

fn main() {
    let text = "привет мир";
    let file_path = "./encoded.json";

    let encoded = encode(text);

    println!("{:?}", encoded);

    let file = File::create(file_path).expect("Failed to create out file");
    serde_json::to_writer_pretty(&file, &encoded
        .iter()
        .map(|byte| format!("{:08b}", byte))
        .collect::<Vec<String>>()
    ).expect("Failed to write out file");

    let file = File::open(file_path).unwrap();
    let encoded: Vec<String> = serde_json::from_reader(&file).unwrap();
    let encoded: Vec<u8> = encoded
        .iter()
        .map(|str| u8::from_str_radix(str, 2).unwrap())
        .collect::<Vec<u8>>();

    let decoded = decode(&encoded);

    println!("{}", decoded);
}
