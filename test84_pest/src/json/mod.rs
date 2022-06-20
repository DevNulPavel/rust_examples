use eyre::WrapErr;
use log::debug;
use pest::Parser;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter, Write},
    fs::read_to_string,
};

#[derive(pest_derive::Parser)]
#[grammar = "ini/rules.pest"]
struct JsonParser;

/// Структурка нашего Json, которая оперирует слайсами на исходную строку
enum JsonValue<'a> {
    Object(HashMap<&'a str, JsonValue<'a>>),
    Array(Vec<JsonValue<'a>>),
    Text(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}

impl Display for JsonValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use JsonValue::*;
        match self {
            Object(map) => {
                // Начало объекта
                f.write_char('{')?;

                // Сохраняем размер для проверки добавления запятой
                let items_len = map.len();

                // Идем поэлементно по всем ключам и значениям,
                // записывая в результат ключи и значения
                for (index, (key, value)) in map.iter().enumerate() {
                    // Добавляем строку
                    write!(f, "\"{}\":{}", key, value)?;

                    // Если еще не дошли до самого конца, тогда добавляем запятую
                    if index < items_len - 1 {
                        f.write_char(',')?;
                    }
                }

                // Конечное закрытие объекта
                f.write_char('}')?;
            }
            Array(vec) => {
                // Начало объекта
                f.write_char('[')?;

                // Сохраняем размер для проверки добавления запятой
                let items_len = vec.len();

                // Итерируем по всем элементам массива
                for (index, value) in vec.iter().enumerate() {
                    // Добавляем строку
                    write!(f, "{}", value)?;
                    // Если еще не дошли до самого конца, тогда добавляем запятую
                    if index < items_len - 1 {
                        f.write_char(',')?;
                    }
                }
                // Конечное закрытие объекта
                f.write_char(']')?;
            }
            Text(string) => write!(f, "\"{}\"", string)?,
            Number(number) => write!(f, "{}", number)?,
            Boolean(boolean) => write!(f, "{}", boolean)?,
            Null => f.write_str("null")?,
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub fn parse_json() {
    // Читаем полностью данные
    let file_content = read_to_string("test_data/sample_data.ini").unwrap();

    // Парсим файлик, получаем сразу же его содержимое для анализа
    let parsed_content = JsonParser::parse(Rule::file, &file_content)
        .unwrap()
        .next()
        .unwrap();
}
