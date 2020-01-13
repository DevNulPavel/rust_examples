use Token::*;

use crate::error::Error;

// Описываем тип токена, который надо обработать
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token {
    OperatorPlus,
    OperatorMinus,
    OperatorMultiply,
    OperatorDivide,
    LeftParentheses,
    RightParentheses,
    Integer(i64),
    Start,
    Eof,
}

// Описываем наш сканнер
pub struct Scanner<'a> {
    chars: &'a [u8],
    position: usize,
}

// Методы сканнера
impl<'a> Scanner<'a> {
    // Новый сканнер для выражения
    pub fn new(expression: &str) -> Scanner {
        Scanner {
            chars: expression.as_bytes(),
            position: 0,
        }
    }

    // Переход к следующему токену
    pub fn next_token(&mut self) -> Result<Token, Error> {
        // В цикле пропускаем пробелы
        self.skip_whitespaces();

        // Дошли до конца - конец
        if self.position >= self.chars.len() {
            return Ok(Eof);
        }

        // Читаем новый символ
        let ch = self.chars[self.position];
        // Смещаем позицию
        self.position += 1;

        let token = match ch {
            // Конвертируем тип символа в нужный нам токен
            b'+' => OperatorPlus,
            b'-' => OperatorMinus,
            b'*' => OperatorMultiply,
            b'/' => OperatorDivide,
            b'(' => LeftParentheses,
            b')' => RightParentheses,
            // Если у нас число, значит надо сконвертировать его дальше
            b'0'..=b'9' => {
                // Массив символов числа
                let mut digits = vec![ch as char];

                // Пока встречаются числа - добавляем их в массив чисел
                while self.position < self.chars.len()
                    && self.chars[self.position] >= b'0'
                    && self.chars[self.position] <= b'9'
                {
                    digits.push(self.chars[self.position] as char);
                    self.position += 1
                }

                // Превращаем в строку сборкой символов в кучу и кастом в строку
                let str: String = digits.into_iter().collect();
                // Парсим число
                Integer(str.parse::<i64>().unwrap())
            }
            // Какая-то ошибка
            _ => {
                return Err(Error::UnexpectedChar(ch));
            }
        };
        Ok(token)
    }

    // Пропуск пробелов
    fn skip_whitespaces(&mut self) {
        while self.position < self.chars.len() && self.chars[self.position] == ' ' as u8 {
            self.position += 1
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::*;

    #[test]
    fn test_scanner() {
        let mut scanner = Scanner::new(" 123 + 2 ");
        assert_eq!(Integer(123), scanner.next_token().unwrap());
        assert_eq!(OperatorPlus, scanner.next_token().unwrap());
        assert_eq!(Integer(2), scanner.next_token().unwrap());
        assert_eq!(Eof, scanner.next_token().unwrap());
    }

    #[test]
    fn test_scanner_empty() {
        let mut scanner = Scanner::new("");
        assert_eq!(Eof, scanner.next_token().unwrap())
    }

    #[test]
    fn test_scanner_unexpected_char() {
        let mut scanner = Scanner::new("fck");
        let result = scanner.next_token();
        assert!(result.is_err())
    }
}
