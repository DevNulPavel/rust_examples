use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub errors);

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Calculator6Error {
    InputTooBig,
    OddNumber,
}

pub fn test_errors() {
    assert!(errors::ExprsParser::new().parse("2147483648").is_err());
}
