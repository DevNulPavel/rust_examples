use log::debug;
use pest::Parser;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(pest_derive::Parser)]
#[grammar = "ini/rules.pest"]
struct IniParser;

struct IniFile {
    root_values: HashMap<String, String>,
    sections: HashMap<String, HashMap<String, String>>,
}

#[allow(dead_code)]
pub fn parse_ini() {
    // Читаем полностью данные
    let file_content = read_to_string("test_data/sample_data.ini").unwrap();

    // Парсим файлик, получаем сразу же его содержимое для анализа
    let parsed_content = IniParser::parse(Rule::file, &file_content)
        .unwrap()
        .next()
        .unwrap();
}
