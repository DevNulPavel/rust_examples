mod basic;
mod calculator;
mod ast;

fn main() {
    basic::test_basic();
    calculator::test_calculator();
    ast::test_ast();
    
    // Continue:
    // http://lalrpop.github.io/lalrpop/tutorial/006_macros.html
}
