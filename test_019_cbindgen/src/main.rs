mod my_test_functions;

// use crate::my_test_functions;

fn main(){
    for _ in 0..1000000 {
        my_test_functions::test_icmp();
    }
}