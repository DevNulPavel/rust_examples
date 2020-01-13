extern crate maplit;

use crate::error::Error;
use crate::error::Error::*;
use crate::expression::Expression;
use crate::expression::Expression::*;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::Token::*;

// Функция парсинга
pub fn parse(input: &str) -> Result<Expression, Error> {
    Parser::new(input).parse()
}

// Структура парсера
struct Parser<'a> {
    scanner: Scanner<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    // Создаем новый парсер для строки
    fn new(expression: &str) -> Parser {
        Parser {
            // Создаем сканнер для выражения
            scanner: Scanner::new(expression),
            // Текущая точка - начало
            current_token: Start,
        }
    }

    // Парсим
    fn parse(&mut self) -> Result<Expression, Error> {
        // Переходим к первому токену, пропуская пробелы
        self.next_token()?;

        // Парсим выражение
        let expression = self.parse_expression()?;

        // Если токен - конец строки - можно выводить результат
        // Если нет - значит ошибка
        match self.current_token {
            Eof => Ok(expression),
            _ => Err(UnexpectedToken(self.current_token)),
        }
    }

    fn next_token(&mut self) -> Result<(), Error> {
        self.current_token = self.scanner.next_token()?;
        Ok(())
    }

    fn parse_expression(&mut self) -> Result<Expression, Error> {
        // Пробуем распарсить выражение сложения
        self.parse_additive_expression()
    }

    fn parse_additive_expression(&mut self) -> Result<Expression, Error> {
        // Пробуем распарсить выражение умножения
        let operand = self.parse_multiplicative_expression()?;
        match self.current_token {
            OperatorPlus => {
                self.next_token()?;
                let right_operand = self.parse_additive_expression()?;
                return Ok(Add(Box::new(operand), Box::new(right_operand)));
            }
            OperatorMinus => {
                self.next_token()?;
                let right_operand = self.parse_additive_expression()?;
                return Ok(Subtract(Box::new(operand), Box::new(right_operand)));
            }
            _ => Ok(operand),
        }
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression, Error> {
        // Пробуем распарсить унарную операцию
        let operand = self.parse_unary_expression()?;
        // Проверяем, не операция ли умножения или деления
        match self.current_token {
            OperatorMultiply => {
                self.next_token()?;
                // Парсим правое выражение
                let right_operand = self.parse_multiplicative_expression()?;
                // Создаем операцию умножения
                return Ok(Multiply(Box::new(operand), Box::new(right_operand)));
            }
            OperatorDivide => {
                self.next_token()?;
                let right_operand = self.parse_multiplicative_expression()?;
                return Ok(Divide(Box::new(operand), Box::new(right_operand)));
            }
            _ => Ok(operand),
        }
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, Error> {
        match self.current_token {
            // Если текущий токен - знак +, 
            OperatorPlus => {
                self.next_token()?;
                return self.parse_parenthesized_expression();
            }
            // Если текущий токен - знак -, 
            OperatorMinus => {
                self.next_token()?;
                let operand = self.parse_parenthesized_expression()?;
                return Ok(UnaryMinus(Box::new(operand)));
            }
            // Парсинг скобочного выражения, или просто число, если оно там
            _ => self.parse_parenthesized_expression(),
        }
    }

    // Парсинг скобочного выражения
    fn parse_parenthesized_expression(&mut self) -> Result<Expression, Error> {
        // Смотрим токен
        match self.current_token {
            // Если это левая круглая скобка
            LeftParentheses => {
                // Получаем новый токен
                self.next_token()?;
                // Парсим подвыражение
                let sub_expression = self.parse_expression()?;
                // Если у нас не закрывающаяся скобка - значит ошибка
                if self.current_token != RightParentheses {
                    return Err(UnexpectedToken(self.current_token));
                }
                // Снова следующий токен
                self.next_token()?;
                // Возвращаем дочернее выражение
                return Ok(sub_expression);
            }
            // Если токен - это просто число, то возращаем его
            Integer(value) => {
                self.next_token()?;
                return Ok(Value(value));
            }
            // Иначе ошибка
            _ => Err(UnexpectedToken(self.current_token)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use maplit::*;

    use crate::parser::*;

    #[test]
    fn test_parser() {
        // Описываем возможые варианты и результаты
        let test_cases = hashmap!(
            "  1235" => "1235",
            "(1235)" => "1235",
            "-1" => "-1",
            "-(154)" => "-154",
            "2*1" => "(2*1)",
            "2000*(-1)" => "(2000*-1)",
            "2/(-1)" => "(2/-1)",
            "2*3*-4" => "(2*(3*-4))",
            "5*(2+3)-2*5" => "((5*(2+3))-(2*5))",
            "-(2+3)*6" => "(-(2+3)*6)",
            "10*20+5" => "((10*20)+5)",
            "10+20*5" => "(10+(20*5))",
        );

        // Итерируемся по значениям и смотрим результаты
        for (input, expected) in test_cases.iter() {
            let result = parse(input).unwrap();
            // Результаты должны быть валидными
            assert_eq!(expected, &result.to_string());
        }
    }

    #[test]
    fn test_parser_error() {
        // Описываем возможные ошибочные варианты
        let errors = vec!["1+2+", "1+*2", "++2", "3//3", "(53+4"];
        for input in errors {
            assert!(parse(input).is_err());
        }
    }

}
