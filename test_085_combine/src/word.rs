use combine::{
    many1,
    parser::char::{letter, space},
    sep_by, Parser,
};


pub fn word() {
    // Слово состоит из множества символов
    let word = many1(letter());

    // Слова разделены у нас пробелами
    // После парсинга слов извлекаем самое последнее слово
    let mut parser = sep_by(word, space()).map(|mut words: Vec<String>| words.pop());

    // Выбираем слово последнее с помощью парсинга
    let result = parser.parse("Pick up that word!");

    // Проверка
    assert_eq!(result, Ok((Some("word".to_string()), "!")));
}