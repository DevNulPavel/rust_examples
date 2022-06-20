use log::debug;
use pest::Parser;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(pest_derive::Parser)]
#[grammar = "ini/rules.pest"]
struct IniParser;

type Key = String;
type Value = Option<String>;

#[derive(Default, Debug)]
struct IniFile {
    root_values: HashMap<Key, Value>,
    sections: HashMap<Key, HashMap<Key, Value>>,
}

#[allow(dead_code)]
pub fn parse_ini() {
    // Читаем полностью данные
    let file_content = read_to_string("test_data/sample.ini").unwrap();

    // Парсим файлик, получаем сразу же его содержимое для анализа
    let parsed_content = IniParser::parse(Rule::file, &file_content)
        .unwrap()
        .next()
        .unwrap();

    let mut ini_data = IniFile::default();

    // Ссылка не активную заполняемую хешмапу
    // Заполнение начинаем с корневой хешмапы
    let mut active_hash: &mut HashMap<Key, Value> = &mut ini_data.root_values;

    for line_pair in parsed_content.into_inner() {
        match line_pair.as_rule() {
            Rule::section => {
                // Получаем итератор по ключу и значению
                let mut pair_iter = line_pair.into_inner();

                // Больше не нужно это делать так как мы определили специальное правило с "_"
                // в самом начале. Тем самым у нас будут пропускаться символы при парсинге
                //
                // Пропускаем возможный пробел перед именем секции
                // while pair_iter.peek().map(|p| p.as_rule()) != Some(Rule::key) {
                //     log::trace!("Skip space at section name");
                //     pair_iter.next();
                // }

                // Получаем имя секции
                let section_name = pair_iter.next().unwrap().as_str().to_owned();

                // Затем получаем мутабельную ссылку на хешмапу для этой секции
                // TODO: Можно было бы еще проверить наличие уже секции с таким именем
                // на случай если пользователь попытается повторно объявить ее
                active_hash = ini_data.sections.entry(section_name).or_default();
            }
            Rule::property => {
                // Получаем итератор по ключу и значению
                let mut key_val_pair_iter = line_pair.into_inner();

                // Ключ у нас есть всегда, поэтому смело вызываем .next().unwrap()
                let key_pair = key_val_pair_iter.next().unwrap();
                assert!(key_pair.as_rule() == Rule::key);
                let key = key_pair.as_str().to_owned();

                // Больше не нужно это делать так как мы определили специальное правило с "_"
                // в самом начале. Тем самым у нас будут пропускаться символы при парсинге
                //
                // Пропускаем возможный пробел между ключем и значением
                // while key_val_pair_iter.peek().map(|p| p.as_rule()) != Some(Rule::value) {
                //     log::trace!("Skip space at section key values");
                //     key_val_pair_iter.next();
                // }

                // А вот значение уже не всегда может быть
                let value_pair = key_val_pair_iter.next();
                // Дополнительно сделаем проверку типа правила
                let value = match value_pair {
                    Some(value_pair) => {
                        assert!(value_pair.as_rule() == Rule::value);
                        Some(value_pair.as_str().to_owned())
                    }
                    None => None,
                };

                // Заполняем хешмапу
                active_hash.insert(key, value);
            }
            _ => {}
        }
    }

    debug!("Result ini: {:?}", ini_data);
}
