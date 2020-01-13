extern crate maplit;

use crate::error::Error;
use crate::error::Error::*;
use crate::expression::Expression;
use crate::expression::Expression::*;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::Token::*;

pub fn parse(input: &str) -> Result<Expression, Error> {
    Parser::new(input).parse()
}

struct Parser<'a> {
    scanner: Scanner<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    fn new(expression: &str) -> Parser {
        Parser {
            scanner: Scanner::new(expression),
            current_token: Start,
        }
    }

    fn parse(&mut self) -> Result<Expression, Error> {
        self.next_token()?;
        let expression = self.parse_expression()?;

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
        self.parse_additive_expression()
    }

    fn parse_additive_expression(&mut self) -> Result<Expression, Error> {
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
        let operand = self.parse_unary_expression()?;
        match self.current_token {
            OperatorMultiply => {
                self.next_token()?;
                let right_operand = self.parse_multiplicative_expression()?;
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
            OperatorPlus => {
                self.next_token()?;
                return self.parse_parenthesized_expression();
            }
            OperatorMinus => {
                self.next_token()?;
                let operand = self.parse_parenthesized_expression()?;
                return Ok(UnaryMinus(Box::new(operand)));
            }
            _ => self.parse_parenthesized_expression(),
        }
    }

    fn parse_parenthesized_expression(&mut self) -> Result<Expression, Error> {
        match self.current_token {
            LeftParentheses => {
                self.next_token()?;
                let sub_expression = self.parse_expression()?;
                if self.current_token != RightParentheses {
                    return Err(UnexpectedToken(self.current_token));
                }
                self.next_token()?;
                return Ok(sub_expression);
            }
            Integer(value) => {
                self.next_token()?;
                return Ok(Value(value));
            }
            _ => Err(UnexpectedToken(self.current_token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use maplit::*;

    use crate::parser::*;

    #[test]
    fn test_parser() {
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

        for (input, expected) in test_cases.iter() {
            let result = parse(input).unwrap();
            assert_eq!(expected, &result.to_string());
        }
    }

    #[test]
    fn test_parser_error() {
        let errors = vec!["1+2+", "1+*2", "++2", "3//3", "(53+4"];
        for input in errors {
            assert!(parse(input).is_err());
        }
    }

}
