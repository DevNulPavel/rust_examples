use clap::Parser;
use hand_made_rspt::{
    all_ngram_combinations, clear_text, count_ngram, ngram_chances, ngram_chi, ngram_entropy,
    ONLY_RUSSIAN, ONLY_RUSSIAN_WITH_SPACE, RUSSIAN_ALPHABET,
};
use regex::Regex;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_text_file: String,

    #[arg(short, long, default_value_t = 2)]
    gram_len: usize,
}

/// Перебор всех файлов в директории для сбора статистики
fn collect_statistics(dir: &str, gram_len: usize, result: &mut HashMap<String, usize>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() || path.extension().unwrap() != "txt" {
            continue;
        }

        println!(
            "Processing: {}",
            path.file_name().unwrap().to_str().unwrap()
        );

        let mut file = File::open(path).expect("Failed to open file");
        let mut text = String::new();
        file.read_to_string(&mut text).expect("Failer to read file");

        // Очистка текста от ненужных символов
        // Останутся только буквы алфавита и единичные пробелы
        let regex = Regex::new(ONLY_RUSSIAN_WITH_SPACE).unwrap();
        clear_text(&mut text, &regex);

        count_ngram(&text, gram_len, result);
    }
}

fn main() {
    let args = Args::parse();
    let gram_len = args.gram_len;
    let file_path = &args.input_text_file;

    let mut count: HashMap<String, usize> = HashMap::new();
    collect_statistics("texts", gram_len, &mut count);

    let mut chances: HashMap<String, f32> = HashMap::new();
    ngram_chances(&count, &mut chances);

    let entropy = ngram_entropy(&chances);

    let mut file = File::open(file_path).expect("Failed to open file");
    let mut text = String::new();
    file.read_to_string(&mut text).expect("Failer to read file");

    let regex = Regex::new(r"[^а-яА-ЯёЁ]").unwrap();
    clear_text(&mut text, &regex);

    let mut real_count: HashMap<String, usize> = HashMap::new();
    count_ngram(&text, gram_len, &mut real_count);

    let expected = chances.clone();
    let chi = ngram_chi(&expected, &real_count);

    println!("Entropy: {:.16}", entropy);
    println!("Chi: {:.16}", chi);

    // Добавление комбинаций, которые не встретились в реальных текстах
    if gram_len < 3 {
        let mut combinations: Vec<String> = vec![];
        all_ngram_combinations(RUSSIAN_ALPHABET, args.gram_len, &mut combinations);
        for k in combinations {
            count.entry(k).or_insert(0);
        }
    }

    // #### JSON write to file

    let file_path = format!("statistics/out_ngram_len_{}.json", gram_len);
    let file = File::create(file_path).expect("Failed to create out file");
    serde_json::to_writer_pretty(&file, &count).expect("Failed to write out file");

    let file_path = format!("statistics/out_ngram_chances_{}.json", gram_len);
    let file = File::create(file_path).expect("Failed to create out file");
    serde_json::to_writer_pretty(&file, &chances).expect("Failed to write out file");
}
