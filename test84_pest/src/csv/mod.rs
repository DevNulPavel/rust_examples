use pest::Parser;
use std::fs;

#[derive(pest_derive::Parser)]
#[grammar = "csv/rules.pest"]
pub struct CSVParser;

#[allow(dead_code)]
pub fn parse_csv_1() {
    let successful_parse = CSVParser::parse(Rule::field, "-273.15").unwrap();
    println!("{:?}", successful_parse);

    let unsuccessful_parse = CSVParser::parse(Rule::field, "this is not a number").unwrap_err();
    println!("{:?}", unsuccessful_parse);
}

#[allow(dead_code)]
pub fn parse_csv_2() {
    // Сначала мы читаем файлик
    let unparsed_file = fs::read_to_string("test_data/numbers.csv").expect("cannot read file");

    // Затем пытаемся распарсить
    let parsed = CSVParser::parse(Rule::file, &unparsed_file)
        .expect("Cannot parse file")
        .next()
        .expect("Cannot find data");

    // Обходим записи, которые распарсили
    let mut row_number = 0;
    for record in parsed.into_inner() {
        // Определяем правило, которое соответствует записи
        match record.as_rule() {
            // Если это отдельная запись в строке
            Rule::record => {
                row_number += 1;
                print!("Row {}:\n- fields: ", row_number);

                // Затем обходим значения в строке
                let mut sum = 0.0;
                for field in record.into_inner() {
                    let field_str = field.as_str();
                    // Парсим значение и суммируем
                    sum += field_str.parse::<f32>().unwrap();
                    print!("{},", field_str);
                }
                println!("\n- sum: {}", sum);
            }
            Rule::EOI => println!("Complete"),
            _ => {}
        }
    }
}
