use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "simple/rules.pest"]
struct SimpleParser;

#[allow(dead_code)]
pub fn parse_simple() {
    {
        let parse_result = SimpleParser::parse(Rule::sum, "1773 + 1362").unwrap();
        // Получаем список токенов, которые смогли распарсить
        // Каждый токен содержит rule + pos
        // - rule - правило, которое соответствует токену
        // - pos - позиция в строке
        // Токены могут быть двух типов - начало и конец, с одинаковыми правилами
        let tokens = parse_result.tokens();
        for token in tokens {
            println!("{:?}", token);
        }
    }

    {
        // Сначала парсим строку
        // Затем из-за того, что пар у нас может быть много, то мы просто получаем первую пару
        // с помощью метода итератора .next()
        let pair = SimpleParser::parse(Rule::enclosed, "(..6472..) and more text")
            .unwrap()
            .next()
            .unwrap();

        // Более удобный - это работа с парами, то есть начало и конец. Подход там следующий:
        // - определяем какое правило соответствует паре
        // - используем пару как строку
        // - либо проверка других подправил, которые есть у данного правила если надо
        assert_eq!(pair.as_rule(), Rule::enclosed);
        assert_eq!(pair.as_str(), "(..6472..)");

        // Специальный метод для получения вложенных пар в текущей паре
        let inner_rules = pair.into_inner();
        println!("{}", inner_rules); // --> [number(3, 7)]
    }

    {
        // Парсим и сразу же получаем итератор на вложенные пары
        let pairs = SimpleParser::parse(Rule::sum, "1773 + 1362")
            .unwrap()
            .next()
            .unwrap()
            .into_inner();

        // Можно склонировать итератор и собрать в кучу числа с помощью парсинга
        let numbers = pairs
            .clone()
            .map(|v| v.as_str().parse::<i32>().unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", numbers); // --> [1773, 1362]

        // Итерируемся по исходным парам и проверяем все типы
        for (found, expected) in pairs.zip(vec!["1773", "1362"]) {
            assert_eq!(Rule::number, found.as_rule());
            assert_eq!(expected, found.as_str());
        }
    }
}
