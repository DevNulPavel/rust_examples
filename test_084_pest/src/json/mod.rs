use log::debug;
use pest::{
    error::{Error as PestError, ErrorVariant},
    iterators::Pair,
    Parser, Span,
};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter, Write},
    fs::read_to_string,
};

#[derive(pest_derive::Parser)]
#[grammar = "json/rules.pest"]
struct JsonParser;

/// Структурка нашего Json, которая оперирует слайсами на исходную строку
#[derive(Debug, Clone)]
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

fn new_pest_error(message: String, span: Span) -> PestError<Rule> {
    PestError::new_from_span(ErrorVariant::CustomError { message }, span)
}

macro_rules! next_or_error {
    ($expr:expr, $text:literal, $span:expr) => {
        $expr
            .next()
            .ok_or_else(|| new_pest_error($text.to_string(), $span))
    };
    ($expr:expr, $text:expr, $span:expr) => {
        $expr.next().ok_or_else(|| new_pest_error($text, $span))
    };
}

fn parse_json_value(pair: Pair<'_, Rule>) -> Result<JsonValue<'_>, PestError<Rule>> {
    // Получаем место
    let root_span = pair.as_span();

    match pair.as_rule() {
        Rule::object => {
            debug!("Begin dictionary");

            // Создаем хешмапу
            let mut result = HashMap::new();

            // Затем идем по значениям и ключам
            for pair in pair.into_inner() {
                // Расположение пары в тексте
                let key_value_span = pair.as_span();

                // Итератор по дочерним элементам
                let mut key_value_iter = pair.into_inner();

                // Ключ
                let key =
                    next_or_error!(key_value_iter, "Must contain key", key_value_span.clone())?.as_str();
                debug!("Value with key: {}", key);

                // Значение
                let value = parse_json_value(next_or_error!(
                    key_value_iter,
                    format!("Must contain value for key {}", key),
                    key_value_span
                )?)?;

                result.insert(key, value);
            }
            Ok(JsonValue::Object(result))
        }
        Rule::array => {
            debug!("Begin array");

            // Создаем массив
            let mut result = Vec::new();

            // Затем идем по значениям и ключам
            for pair in pair.into_inner() {
                let value = parse_json_value(pair)?;
                result.push(value);
            }

            Ok(JsonValue::Array(result))
        }
        Rule::text => {
            // Разворачиваем чтобы опустить кавычки
            let string = next_or_error!(pair.into_inner(), "No string", root_span)?.as_str();
            debug!("String: {}", string);

            Ok(JsonValue::Text(string))
        }
        Rule::number => {
            let val: f64 = pair.as_str().parse().map_err(|err| {
                new_pest_error(
                    format!("Invalid number parsing with err: {}", err),
                    root_span,
                )
            })?;

            Ok(JsonValue::Number(val))
        }
        Rule::boolean => {
            let boolean = pair.as_str() == "true";
            Ok(JsonValue::Boolean(boolean))
        }
        Rule::null => Ok(JsonValue::Null),
        _ => Err(new_pest_error("Unexpected token".to_string(), root_span)),
    }
}

#[allow(dead_code)]
pub fn parse_json() {
    // Читаем полностью данные
    let file_content = read_to_string("test_data/sample.json").unwrap();

    // Парсим файлик, получаем сразу же его содержимое для анализа
    let parsed_content = JsonParser::parse(Rule::file, &file_content)
        .unwrap()
        .next()
        .unwrap();

    let json_data = parse_json_value(parsed_content).unwrap();
    debug!("Parsed json: {}", json_data);
}
