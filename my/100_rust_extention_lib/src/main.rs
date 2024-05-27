mod icmp_function;

// use crate::my_test_functions;

fn main(){
    // Код для профилирования
    for _ in 0..1_000_000 {
        icmp_function::test_icmp();
    }
}