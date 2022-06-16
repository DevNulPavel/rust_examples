use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "ident.pest"]
struct IdentParser;

#[allow(dead_code)]
pub fn parse_ident() {
    // Парсим
    let pairs = IdentParser::parse(Rule::ident_list, "a1 b2").unwrap();

    // Обходим полученные значения
    for pair in pairs {
        // Пара - это комбинация правила, которое соответствует span входных данных
        println!("Pair:");
        println!("- rule: {:?}", pair.as_rule());
        println!("- span: {:?}", pair.as_span());
        println!("- text: {}", pair.as_str());
        //println!("Tokens:  {:?}", pair.tokens());

        // Пара может быть конвертирована в итератор токенов
        println!("- info:");
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::alpha => println!("  - letter: {}", inner_pair.as_str()),
                Rule::digit => println!("  - digit: {}", inner_pair.as_str()),
                _ => unreachable!(),
            };
        }
        // for token in pair.tokens() {
        //     println!("  - Token: {:?}", token);
        // }
    }
}
