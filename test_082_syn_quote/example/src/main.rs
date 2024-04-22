use macroses::trace_var;

#[trace_var(p, n)]
fn factorial(mut n: u64) -> u64 {
    let mut p = 1;
    while n > 1 {
        p *= n;
        n -= 1;
    }
    p
}

fn demo(s: &str) -> &str {
    &s[1..]
}

// #[no_panic::no_panic]
fn possible_panic() {
    println!("{}", demo("input string"));
}

// #[no_panic::no_panic]
fn main() {
    possible_panic();

    //let mut vec = <Vec<i32>>::new();
    //let value = <Option<()> as core::default::Default>::default();

    println!("{}", factorial(8));
}
