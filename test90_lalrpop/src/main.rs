use lalrpop_util::lalrpop_mod;

lalrpop_mod!(basic);

fn main() {
    assert_eq!(basic::TermParser::new().parse("123").unwrap(), 123);
    assert_eq!(basic::TermParser::new().parse("(123)").unwrap(), 123);

    assert!(basic::TermParser::new().parse("((123a))").is_err());
    assert!(basic::TermParser::new().parse("((123)").is_err());
    
    // Continue:
    // http://lalrpop.github.io/lalrpop/tutorial/005_building_asts.html
}
