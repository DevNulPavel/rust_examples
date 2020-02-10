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

fn partition(v: &mut [i32]) -> usize {
    // Опорная точка - конец
    let pivot = v.len() - 1;
    let mut i = 0;
    // Итерируемся от 0 до конца
    for j in 0..pivot {
        // Если текущее значение меньше конечного
        if v[j] <= v[pivot] {
            // Тогда меняем значение первого элемента и текущего
            v.swap(i, j);
            i += 1;
        }
    }
    v.swap(i, pivot);
    i
}

pub fn quick_sort(arr: &mut[i32]) {
    if arr.len() <= 1 {
        return;
    }

    let mid = partition(arr);
    let (lo, hi) = arr.split_at_mut(mid);
    rayon::join(|| quick_sort(lo), || quick_sort(hi));
}

fn main() {
    {
        let vec = vec![1, 2, 3, 4, 5];
        let result = sum_of_squares(&vec);
        println!("{}", result);    
    }

    join_test();

    {
        let mut vec = vec![3, 6, 1, 2, 8, 10, 23];
        quick_sort(&mut vec);
        assert_eq!(vec, [1, 2, 3, 6, 8, 10, 23]);
    }
}
