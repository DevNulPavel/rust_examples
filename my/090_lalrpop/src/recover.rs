use lalrpop_util::lalrpop_mod;
use std::fmt::{Debug, Formatter};

// Здесь в макросе мы можем указывать так же настройки публичности,
// а так же настройки разные для clippy и тд
lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    pub(crate) recover
);

//////////////////////////////////////////////////////////////////////////////////////////

/// Непосредственно выражение
pub(crate) enum Expr {
    /// Просто число, которое мы распарсили
    Number(i32),

    /// Рекурсивное выражение, поэтому используем здесь Box
    Op(Box<Expr>, Opcode, Box<Expr>),

    /// Какая-то ошибка в парсинге токенов
    Error,
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        use self::Expr::*;
        match self {
            Number(n) => write!(fmt, "{:?}", n),
            Op(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            Error => write!(fmt, "error"),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

/*pub(crate) enum ExprSymbol<'input> {
    NumSymbol(&'input str),
    Op(Box<ExprSymbol<'input>>, Opcode, Box<ExprSymbol<'input>>),
    Error,
}

impl<'input> Debug for ExprSymbol<'input> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        use self::ExprSymbol::*;
        match *self {
            NumSymbol(n) => write!(fmt, "{:?}", n),
            Op(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            Error => write!(fmt, "error"),
        }
    }
}*/

//////////////////////////////////////////////////////////////////////////////////////////

/// Операция
#[derive(Copy, Clone)]
pub(crate) enum Opcode {
    Mul,
    Div,
    Add,
    Sub,
}

impl Debug for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        use self::Opcode::*;
        match *self {
            Mul => write!(fmt, "*"),
            Div => write!(fmt, "/"),
            Add => write!(fmt, "+"),
            Sub => write!(fmt, "-"),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn test_recover() {
    // Здесь мы можем создать какой-то наш тип, который будем передавать в парсер
    let mut errors = Vec::new();

    let expr = recover::ExprParser::new()
        .parse(&mut errors, "22 * + 3")
        .unwrap();
    assert_eq!(&format!("{:?}", expr), "((22 * error) + 3)");

    assert_eq!(errors.len(), 1);
}
