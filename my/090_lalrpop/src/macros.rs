use lalrpop_util::lalrpop_mod;

lalrpop_mod!(macros);

//////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn test_macros() {
    assert_eq!(macros::ExprsParser::new().parse("10 + 5 * 10, 10 + 10").unwrap(), [60, 20]);

    assert!(macros::ExprsParser::new().parse("2147483648").is_err());
}
