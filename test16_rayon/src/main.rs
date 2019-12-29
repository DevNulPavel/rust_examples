use std::thread;
use std::time;
use rayon::prelude::*;
use rayon::join;

fn sum_of_squares(input: &[i32]) -> i32 {
    // Мы просто заменяем вызов обычного итератора на итератор параллельный
    input.par_iter().map(|&i| i * i).sum()
}

fn join_test(){
    // `do_something` and `do_something_else` *may* run in parallel
    join(|| {
        thread::sleep(time::Duration::from_millis(1000));
    }, || {
        thread::sleep(time::Duration::from_millis(2000));
    });
}

fn main() {
    let vec = vec![1, 2, 3, 4, 5];
    let result = sum_of_squares(&vec);
    println!("{}", result);

    join_test();
}
