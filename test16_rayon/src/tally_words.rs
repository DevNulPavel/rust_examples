#![warn(clippy::all)]

//extern crate rayon;

use std::collections::HashMap;
use std::env;
use std::fs;
use rayon::prelude::*;
//use std::error::Error;

type Words = HashMap<String, u32>;

///////////////////////////////////////////////////////////////////////////

// Собственный enum ошибки
#[derive(Debug)]
pub enum TallyError{
    FileOpenError(std::io::Error)
}

// Реализация автоматической конвертации
impl From<std::io::Error> for TallyError{
    fn from(err: std::io::Error) -> Self {
        TallyError::FileOpenError(err)
    }
}

///////////////////////////////////////////////////////////////////////////

fn tally_words(filename: &str) -> Result<Words, TallyError> {
    // Создаем словарь
    let mut words = Words::new();
    // Читаем файлик в строку
    let contents = fs::read_to_string(filename)?;

    // Разделяем по пробелам
    for s in contents.split_whitespace() {
        // Для каждого слова суммируем количество
        let key = s.to_lowercase();
        *words.entry(key).or_insert(0) += 1;
    }
    Ok(words)
}

pub fn test_tally() -> Result<(), TallyError> {
    let words = env::args()
        // Пропускаем первый аргумент
        .skip(1)
        // Собираем значения в вектор
        .collect::<Vec<String>>()
        // Создаем параллельний итератор
        .par_iter()
        // Код выполняется параллельно
        .map(|arg| {
            tally_words(arg)
                .map_err(|e| {
                    // Печатаем ошибку в stderr, а не в stdout
                    eprintln!("Error processing {}: {:?}", arg, e);
                })
                // Либо получаем значение, либо выдаем дефолтное, но не падаем
                .unwrap_or_default()
        })
        // Затем упаковываем в один словарь
        .reduce(
            || {
                Words::new()
            },
            |mut result, current| {
                for (key, val) in current {
                    // Берем и добавляем новый элемент
                    result
                        .entry(key)
                        .and_modify(|e| {
                            *e += val;
                        })
                        .or_insert(val);
                }
                result
            }
        );

    // Выводим результат
    for (word, count) in words.iter() {
        if *count > 1 {
            println!("{} {}", count, word)
        }
    }

    Ok(())
}