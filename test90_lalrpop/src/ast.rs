use std::fmt::{Debug, Formatter};

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(ast);

//////////////////////////////////////////////////////////////////////////////////////////

/// Непосредственно выражение
pub(crate) enum Expr {
    /// Просто число, которое мы распарсили
    Number(i32),

    /// Рекурсивное выражение, поэтому используем здесь Box
    Op(Box<Expr>, Opcode, Box<Expr>),
    // Error,
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        use self::Expr::*;
        match self {
            Number(n) => write!(fmt, "{:?}", n),
            Op(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            // Error => write!(fmt, "error"),
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

pub(crate) fn test_ast() {
    let expr = ast::ExprParser::new().parse("22 * 44 + 66").unwrap();
    assert_eq!(&format!("{:?}", expr), "((22 * 44) + 66)");
}
