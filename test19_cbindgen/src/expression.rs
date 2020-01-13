use std::fmt;

// Можно использовать директиву use даже для кода ниже
//use Expression::*;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    UnaryMinus(Box<Expression>),
    Value(i64),
}

impl fmt::Display for Expression {
    // Реализуем вывод форматирования для отладки
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Value(value) => {
                write!(f, "{}", value)
            },
            Self::Add(left, right) => {
                write!(f, "({}+{})", left, right)
            },
            Self::Subtract(left, right) => {
                write!(f, "({}-{})", left, right)
            },
            Self::UnaryMinus(operand) => {
                write!(f, "-{}", operand)
            },
            Self::Multiply(left, right) => {
                write!(f, "({}*{})", left, right)
            },
            Self::Divide(left, right) => {
                write!(f, "({}/{})", left, right)
            },
        }
    }
}
