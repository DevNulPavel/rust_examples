mod basic;
mod calculator;
mod ast;
mod macros;
mod errors;
mod recover;

fn main() {
    basic::test_basic();
    calculator::test_calculator();
    ast::test_ast();
    macros::test_macros();
    errors::test_errors();
    recover::test_recover();
    
    // Continue:
    // http://lalrpop.github.io/lalrpop/tutorial/009_state_parameter.html
}
