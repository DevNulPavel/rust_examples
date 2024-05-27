use lalrpop_util::lalrpop_mod;

lalrpop_mod!(basic);

pub(crate) fn test_basic() {
    assert_eq!(basic::TermParser::new().parse("123").unwrap(), 123);
    assert_eq!(basic::TermParser::new().parse("(123)").unwrap(), 123);

    assert!(basic::TermParser::new().parse("((123a))").is_err());
    assert!(basic::TermParser::new().parse("((123)").is_err());
}
