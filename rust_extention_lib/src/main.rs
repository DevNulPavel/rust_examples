mod icmp_function;

// use crate::my_test_functions;

fn main(){
    // Код для профилирования
    for _ in 0..1000000 {
        icmp_function::test_icmp();
    }
}