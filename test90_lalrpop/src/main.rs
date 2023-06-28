use lalrpop_util::lalrpop_mod;

lalrpop_mod!(syntax);

fn main() {
    assert_eq!(syntax::TermParser::new().parse("123").unwrap(), 123);
    assert_eq!(syntax::TermParser::new().parse("(123)").unwrap(), 123);

    assert!(syntax::TermParser::new().parse("((123a))").is_err());
    assert!(syntax::TermParser::new().parse("((123)").is_err());
    
    // Continue:
    // http://lalrpop.github.io/lalrpop/tutorial/004_full_expressions.html
}
