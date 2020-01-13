use Token::*;

use crate::error::Error;

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

pub struct Scanner<'a> {
    chars: &'a [u8],
    position: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(expression: &str) -> Scanner {
        Scanner {
            chars: expression.as_bytes(),
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, Error> {
        self.skip_whitespaces();

        if self.position >= self.chars.len() {
            return Ok(Eof);
        }

        let ch = self.chars[self.position];
        self.position += 1;
        let token = match ch {
            b'+' => OperatorPlus,
            b'-' => OperatorMinus,
            b'*' => OperatorMultiply,
            b'/' => OperatorDivide,
            b'(' => LeftParentheses,
            b')' => RightParentheses,
            b'0'..=b'9' => {
                let mut digits = vec![ch as char];

                while self.position < self.chars.len()
                    && self.chars[self.position] >= b'0'
                    && self.chars[self.position] <= b'9'
                {
                    digits.push(self.chars[self.position] as char);
                    self.position += 1
                }

                let str: String = digits.into_iter().collect();
                Integer(str.parse::<i64>().unwrap())
            }
            _ => {
                return Err(Error::UnexpectedChar(ch));
            }
        };
        Ok(token)
    }

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
