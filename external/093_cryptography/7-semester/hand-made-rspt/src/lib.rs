use std::collections::HashMap;

use regex::Regex;

pub const RUSSIAN_ALPHABET: &[char] = &['а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с', 'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я'];

pub const ONLY_RUSSIAN: &str = r"[^а-яА-ЯёЁ]";
pub const ONLY_RUSSIAN_WITH_SPACE: &str = r"[^а-яА-ЯёЁ\s]";
pub const ONLY_RUSSIAN_WITH_PUNCTUATION: &str = r"[^а-яА-ЯёЁ\.\,]";

pub fn clear_text(text: &mut String, regex: &Regex) {
    *text = regex.replace_all(text, "").to_string();
    let regex = Regex::new(r"\n+|\s{2,}").unwrap();
    *text = regex.replace_all(text, " ").to_string();
    let regex = Regex::new(r"\.{2,}").unwrap();
    *text = regex.replace_all(text, ".").to_string();
    let regex = Regex::new(r"\,{2,}").unwrap();
    *text = regex.replace_all(text, " ").to_string();
    *text = text.to_lowercase().trim().to_string();
}

/// # Перечисляет все возможные комбинации н-грамм
pub fn all_ngram_combinations(alphabet: &[char], ngram_len: usize, combinations: &mut Vec<String>) {
    let comb_num = alphabet
        .len()
        .checked_pow(ngram_len as u32)
        .expect("Too many combinations");

    let mut combination: Vec<char> = vec![alphabet[0]; ngram_len];

    combinations.push(combination.iter().collect());

    for i in 1..comb_num {
        for (j, c) in combination.iter_mut().enumerate() {
            if *c == alphabet[alphabet.len() - 1] {
                *c = alphabet[0];
            } else {
                let next_letter_index = (i / alphabet.len().pow(j as u32)) % alphabet.len();
                *c = alphabet[next_letter_index];
                break;
            }
        }
        combinations.push(combination.iter().collect());
    }
}

/// # Считает количество вхождений н-грамм в текст
pub fn count_ngram(
    text: &str,
    ngram_len: usize,
    result: &mut HashMap<String, usize>
) {
    let text: Vec<char> = text.chars().collect();
    for i in 0..text.len() - ngram_len {
        let ngram: String = text[i..i + ngram_len].iter().collect();
        *result.entry(ngram).or_insert(0) += 1;
    }
}

/// # Считает вероятности появления н-грамм
pub fn ngram_chances(
    ngrams: &HashMap<String, usize>,
    result: &mut HashMap<String, f32>
) {
    let mut total_ngrams = 0;
    for v in ngrams.values() {
        total_ngrams += v;
    }

    for (k, v) in ngrams {
        result.insert(k.to_string(), *v as f32 / total_ngrams as f32);
    }
}

/// # Считает энтропию
pub fn ngram_entropy(
    ngram_chances: &HashMap<String, f32>
) -> f32 {
    let mut entropy = 0f32;
    for v in ngram_chances.values() {
        entropy -= v * v.log2();
    }
    entropy
}

/// # Считает характеристику Хи
pub fn ngram_chi(
    chances: &HashMap<String, f32>,
    count: &HashMap<String, usize>,
) -> f32 {
    let mut chi = 0.0;
    let mut total_count = 0;
    for v in count.values() {
        total_count += v;
    }

    for (k, v) in chances {
        let np = v * total_count as f32;

        chi += (*count.get(k).unwrap_or(&0) as f32 - np).powf(2.0) / np;
    }

    chi
}
