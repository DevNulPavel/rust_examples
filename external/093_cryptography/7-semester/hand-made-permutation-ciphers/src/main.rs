use std::io::Write;
use std::{collections::HashMap, fs::File, io::Read};
use std::collections::hash_map::Entry;

const ENC_FILE_PATH: &str = "enc_texts/08.txt";
const DEC_FILE_PATH: &str = "dec_texts/08.txt";
const EXCLUDED_FILE_PATH: &str = "statistics/excluded_ngrams.json";

/// Проверяет столбцы на наличие исключенных n-грамм и заполняет результирующую карту.
///
/// # Аргументы
///
/// * `columns` - Вектор столбцов символов
/// * `excluded` - Список исключенных n-грамм
/// * `result` - Результирующая карта, где ключ - индекс столбца, а значение - вектор индексов столбцов, с которыми он образует исключенные n-граммы
fn check_columns(
    columns: &[Vec<char>],
    excluded: &[String],
    result: &mut HashMap<usize, Vec<usize>>
) {
    for i in 0..columns.len() {
        for j in 0..columns.len() {
            if i == j {
                continue;
            }

            for k in 0..columns[0].len() {
                let old_char = columns[i][k];
                let new_char = columns[j][k];
                let ngram = String::from_iter([old_char, new_char]);

                if excluded.contains(&ngram) && !result.entry(i).or_insert_with(|| {
                    Vec::with_capacity(columns.len() - 1)
                }).contains(&j) {
                    result.get_mut(&i).unwrap().push(j);
                }
            }
        }
    }
}

/// Отображает матрицу исключений в удобочитаемом формате.
///
/// # Аргументы
///
/// * `exclude` - HashMap, где ключ - индекс столбца, а значение - вектор индексов столбцов,
///               с которыми он образует исключенные n-граммы.
///
/// Функция выводит матрицу, где 'XX' обозначает исключенную пару, а '__' - допустимую.
fn display_exclude(
    exclude: &HashMap<usize, Vec<usize>>
) {
    for i in 0..exclude.len() {
        if i == 0 {
            print!("    {:2} ", i);
        } else {
            print!("{:2} ", i);
        }
    }
    println!();
    for i in 0..exclude.len() {
        print!("{:2}: ", i);
        for j in 0..exclude.len() {
            if i == j {
                print!("XX ");
                continue;
            }
            if exclude.get(&j).unwrap().contains(&i) {
                print!("XX ");
            } else {
                print!("__ ");
            }
        }
        println!();
    }
}

/// Строит порядок столбцов на основе исключений.
///
/// # Аргументы
///
/// * `exclude` - HashMap с исключенными парами столбцов
///
/// # Возвращаемое значение
///
/// `Option<Vec<usize>>` - Порядок столбцов или None, если порядок невозможно построить
fn get_order(
    exclude: &HashMap<usize, Vec<usize>>
) -> Option<Vec<usize>> {
    let mut order_map = HashMap::new();

    for i in 0..exclude.len() {
        for j in 0..exclude.len() {
            if i == j || exclude.get(&j).unwrap().contains(&i) {
                continue;
            }

            if let Entry::Vacant(e) = order_map.entry(i) {
                e.insert(j);
            } else {
                return None;
            }
        }
    }

    let mut order: Vec<usize> = Vec::with_capacity(exclude.len());
    let mut last_index: Option<usize> = None;

    for v in order_map.values() {
        if !order_map.contains_key(v) {
            match last_index {
                None => last_index = Some(*v),
                Some(_) => return None,
            }
        }
    }

    let mut last_index: usize = last_index.unwrap();
    order_map = order_map.iter().map(|(&k, &v)| (v, k)).collect();

    order.push(last_index);

    for _ in 0..order_map.len() {
        let v = *order_map.get(&last_index).unwrap();
        order.push(v);
        last_index = v;
    }

    Some(order)
}

fn main() {
    let mut file = File::open(ENC_FILE_PATH).expect("Failed to open file");
    let mut text = String::new();
    file.read_to_string(&mut text).expect("Failer to read file");

    let text = text
        .to_lowercase()
        .trim()
        .replace("_", " ");
    let strings: Vec<&str> = text.split("\r\n").collect();

    for i in 1..strings.len() {
        if strings[i-1].chars().count() != strings[i].chars().count() {
            panic!("Количество колонок не равное! {}:{}", i, i+1);
        }
    }

    let mut columns: Vec<Vec<char>> = vec![Vec::new(); strings[0].chars().count()];
    for s in strings.iter() {
        let string: Vec<char> = s.chars().collect();
        for (j, c) in string.iter().enumerate() {
            columns[j].push(*c);
        }
    }

    let file = File::open(EXCLUDED_FILE_PATH).expect("Failed to open file");
    let excluded: Vec<String> = serde_json::from_reader(file).unwrap();

    let mut exclude: HashMap<usize, Vec<usize>> = HashMap::new();
    check_columns(&columns, &excluded, &mut exclude);

    display_exclude(&exclude);

    let order = get_order(&exclude).unwrap();
    let mut text = String::with_capacity(columns[0].len());

    for i in 0..columns[0].len() {
        let mut string = String::with_capacity(columns.len());
        for j in &order {
            string.push(columns[*j][i]);
        }
        text.push_str(&string);
    }

    let mut file = File::create(DEC_FILE_PATH).expect("Failed to open file");
    file.write_all(text.as_bytes()).unwrap();
}
