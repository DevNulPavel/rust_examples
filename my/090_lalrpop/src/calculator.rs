use lalrpop_util::lalrpop_mod;

lalrpop_mod!(calculator);

pub(crate) fn test_calculator() {
    assert_eq!(calculator::ExprParser::new().parse("10 + 5 * 10").unwrap(), 60);
}
