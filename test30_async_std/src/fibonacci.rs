#![warn(clippy::all)]
#![warn(dead_code)]

pub fn fibonacci_recursive(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci_recursive(n-1) + fibonacci_recursive(n-2),
    }
}

pub fn fibonacci_non_recursive(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
pub mod tests {
    #[test]
    fn fibonacci_req_test() {
        use super::fibonacci_non_recursive as test_f;

        assert_eq!(test_f(10), 89);
        assert_eq!(test_f(20), 10946);
        assert_eq!(test_f(1), 1);
    }

    #[test]
    fn fibonacci_non_rec_test() {
        use super::fibonacci_non_recursive as test_f;

        assert_eq!(test_f(10), 89);
        assert_eq!(test_f(20), 10946);
        assert_eq!(test_f(1), 1);
    }
}