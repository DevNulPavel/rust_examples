#![warn(clippy::all)]

mod tally_words;
mod sort;

use std::thread;
use std::time;
use std::sync::mpsc::channel;
use rayon::prelude::*;
use rayon::join;
use rand::Rng;
use tally_words::test_tally;
use sort::quick_sort;


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
    {
        let vec = vec![1, 2, 3, 4, 5];
        let result = sum_of_squares(&vec);
        //println!("{}", result);
        assert_eq!(result, 55);
    }

    join_test();

    {
        let mut vec = vec![3, 6, 1, 2, 8, 10, 23];
        quick_sort(&mut vec);
        assert_eq!(vec, [1, 2, 3, 6, 8, 10, 23]);
    }

    test_tally().unwrap();

    {
        (0..100)
            // Создаем параллельный итератор, который забирает значения (self не по ссылке)
            .into_par_iter()
            // Работает параллельно, порядок будет неправильный
            .for_each(|_x| {
                //println!("{:?}", _x);
            });
    }

    {
        // Создаем канал для передачи
        let (sender, receiver) = channel();
        (0..5)
            // Создаем параллельный итератор, принимающий значения
            .into_par_iter()
            // Можно вызывать для каждого итема функцию + заранее заданный параметр,
            // у которого каждый раз будет вызываться clone
            .for_each_with(sender, |s, x| {
                s.send(x).unwrap()
            });
        
        // Получаем тут значения
        let mut res: Vec<_> = receiver.iter().collect();
        // Сортируем
        res.sort();       
        assert_eq!(&res[..], &[0, 1, 2, 3, 4])
    }

    {
        // Создаем вектор на миллион элементов
        let mut v = vec![0u8; 1_000_000];
        v
            // Создаем параллельный итератор, который делит исходный массив на чанки по 1000 элементов
            .par_chunks_mut(1000)
            // Создаем для каждого потока свой локальный генератор случайных чисел
            .for_each_init(
                // Код будет вызван один раз для каждого потока
                || {
                    rand::thread_rng()
                },
                // Принимает генератор и массив значений, который мы заполняем
                |rng, chunk| {
                    rng.fill(chunk)
                },
            );
        // Низкая вероятность, что из миллиона значений не будет всех возможных
        for i in 0u8..=255 {
            assert!(v.contains(&i));
        }        
    }

    {
        // Параллельная итерация закончится раньше если у нас вернется ошибка
        (0..100)
            .into_par_iter()
            .try_for_each(|_x| {
                //use std::io::{self, Write};
                //return writeln!(io::stdout(), "{:?}", x);
                Result::<(), &str>::Ok(())
            })
            .expect("expected no write errors");
    }

    {
        // Создаем канал для обмена данными
        let (sender, receiver) = channel();
        (0..5)
            .into_par_iter()
            // Прерываем итерирование если возвращается ошибка
            .try_for_each_with(sender, |s, x| { 
                s.send(x)
            })
            .expect("expected no send errors");

        let mut res: Vec<_> = receiver.iter().collect();
        res.sort();
        assert_eq!(&res[..], &[0, 1, 2, 3, 4])
    }

    {
        // Можно посчитать количество итемов в параллельном итераторе
        let count = (0..100).into_par_iter().count();
        assert_eq!(count, 100);        
    }

    {
        // Параллельный итератор, умножающий на 2
        let par_iter = (0..5)
            .into_par_iter()
            .map(|x| x * 2);
        let doubles: Vec<_> = par_iter.collect();
        assert_eq!(&doubles[..], &[0, 2, 4, 6, 8]);    
    }

    {
        // Можем модифицировать передаваемые значения параллельно
        let doubles: Vec<_> = (0..5)
            .into_par_iter()
            .update(|x| {
                *x *= 2;
            })
            .collect();
        assert_eq!(&doubles[..], &[0, 2, 4, 6, 8]);        
    }

    {
        // Iterate over a sequence of pairs `(x0, y0), ..., (xN, yN)`
        // and use reduce to compute one pair `(x0 + ... + xN, y0 + ... + yN)`
        // where the first/second elements are summed separately.

        // Главная разница с методом fold - здесь порядок не специфицирован

        // Итерируемся по парам значений
        let sums = [(0, 1), (5, 6), (16, 2), (8, 9)]
                .par_iter()        // iterating over &(i32, i32)
                .cloned()          // iterating over (i32, i32)
                .reduce(|| {
                        (0, 0) // Начальное значение
                    },
                    |prev, b| {
                        (prev.0 + b.0, prev.1 + b.1)
                    });
        assert_eq!(sums, (5 + 16 + 8, 1 + 6 + 2 + 9));    
    }

    {
        // Главное отличие fold от reduce:
        // - можно работать с другим начальным типом, отличным от итерируемого
        // - ??? сохраняется порядок обработки значений несмотря на параллельность
        let bytes: std::ops::Range<u8> = 0..22_u8;
        let sum = bytes
            .into_par_iter()
            .fold(|| {
                0_u32
            }, |a: u32, b: u8| {
                //println!("{}", b);
                a + (b as u32)
            })
            .sum::<u32>();
        assert_eq!(sum, (0..22).sum()); // compare to sequential        
    }

    {
        let mut temp_res: Vec<(usize, i32)> = Vec::new();

        // Параллельно выполняем работу, но сохраняем индексы у результатов
        (0..6)
            .into_par_iter()
            .enumerate()
            .map(|(i, val)|{
                (i, val * 10)
            })
            .collect_into_vec(&mut temp_res); // Вроде как данный вариант более эффективный чем collect
        
        // !!! Вызов collect сохраняет исходный порядок элементов, 
        // Дополнительно делать ничего не надо
        /*// Параллельная сортировка по индексам
        temp_res.par_sort_by_key(|val|{
            val.0
        });*/

        // Превращаем в массив только значений
        let result: Vec<i32> = temp_res
            .into_iter() // Значение временного результата потребляется
            .map(|val| {
                val.1
            })
            .collect();
        
        assert_eq!(&result, &[0, 10, 20, 30, 40, 50]);
    }
}
