mod basic;
mod calculator;
mod ast;
mod macros;

fn main() {
    basic::test_basic();
    calculator::test_calculator();
    ast::test_ast();
    macros::test_macros();
    
    // Continue:
    // http://lalrpop.github.io/lalrpop/tutorial/007_fallible_actions.html
}
