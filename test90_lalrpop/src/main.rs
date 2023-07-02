mod basic;
mod calculator;
mod ast;
mod macros;
mod errors;

fn main() {
    basic::test_basic();
    calculator::test_calculator();
    ast::test_ast();
    macros::test_macros();
    errors::test_errors();
    
    // Continue:
    // http://lalrpop.github.io/lalrpop/tutorial/007_fallible_actions.html
}
