use std::{collections::HashMap, fs::File, io::{Read, Write}};

use hand_made_rspt::{clear_text, count_ngram, ngram_chances, ONLY_RUSSIAN};
use regex::Regex;

const NGRAM_LEN: usize = 3;
const ENC_FILE_PATH: &str = "enc_texts/5B.txt";
const DEC_FILE_PATH: &str = "dec_texts/5B.txt";
const SOLVED_LETTERS_PATH: &str = "enc_texts/solved_letters_5B.json";

fn main() {
    let mut file = File::open(ENC_FILE_PATH).expect("Failed to open file");
    let mut text = String::new();
    file.read_to_string(&mut text).expect("Failer to read file");

    let regex = Regex::new(ONLY_RUSSIAN).unwrap();
    clear_text(&mut text, &regex);

    let mut count: HashMap<String, usize> = HashMap::new();
    count_ngram(&text, NGRAM_LEN, &mut count);
    let mut count_vec: Vec<(&String, &usize)> = count.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));

    let mut chances: HashMap<String, f32> = HashMap::new();
    ngram_chances(&count, &mut chances);
    let mut chances_vec: Vec<(&String, &f32)> = chances.iter().collect();
    chances_vec.sort_by(|a, b| b.1.total_cmp(a.1));

    let file_path = format!("statistics/out_ngram_chances_{}.json", NGRAM_LEN);
    let file = File::open(&file_path).unwrap_or_else(|e| panic!("Failed to open file: {}\n{}", &file_path, e));
    let statistics: HashMap<String, f32> = serde_json::from_reader(file).unwrap();

    let mut statistics_vec: Vec<(&String, &f32)> = statistics.iter().collect();
    statistics_vec.sort_by(|a, b| b.1.total_cmp(a.1));

    let file = File::open(SOLVED_LETTERS_PATH).expect("Failed to open file");
    let solved: HashMap<char, char> = serde_json::from_reader(file).unwrap();

    println!("Н-граммы:");
    for i in 0..statistics_vec.len().min(chances_vec.len()).min(10) {
        println!(
            "[{:>10}]: {:.08}  ->  [{:>10}]: {:.08}",
            statistics_vec[i].0,
            statistics_vec[i].1,
            chances_vec[i].0,
            chances_vec[i].1,
        );
    }

    let replace_table: HashMap<char, char> = solved.iter().map(|(k, v)| (*v, *k)).collect();
    let mut solved_text: Vec<char> = text.chars().collect();

    for i in solved_text.iter_mut() {
        if replace_table.contains_key(i) {
            *i = *replace_table.get(i).unwrap();
        } else {
            *i = '_';
        }
    }

    let solved_text: String = String::from_iter(solved_text);

    println!("\nПодстановки (расшифрование):");
    for k in replace_table.keys() {
        print!("{} ", k);
    }
    println!();
    for v in replace_table.values() {
        print!("{} ", v);
    }
    println!();

    println!("\nЗашифрованный текст:\n{}", &text);
    println!("\nРасшифрованный текст:\n{}", &solved_text);

    let mut file = File::create(DEC_FILE_PATH).expect("Failed to open file");
    file.write_all(solved_text.as_bytes()).unwrap();
}
