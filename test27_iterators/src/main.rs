macro_rules! map(
    // Макрос вида {key => val, key => val}
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            // TODO: Reserve size
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);



// Структура, являющаяся итератором
struct Counter {
    count: usize,
    max: usize
}

// Счетчик будет начинаться с 0, поэтому конструируем объект
impl Counter {
    fn new(max: usize) -> Counter {
        Counter{ 
            count: 0,
            max: max
        }
    }
}

// Затем мы реализуем метод итерирования для нашего счетчика
impl Iterator for Counter {
    // Содержимое - usize
    type Item = usize;

    // Единственный метод, который мы реализуем
    fn next(&mut self) -> Option<Self::Item> {
        // Увеличиваем на 1, итератор будет начинаться с 1
        self.count += 1;

        // Возвращаем значение
        if self.count <= self.max {
            Some(self.count)
        } else {
            None
        }
    }

    // Мы можем самостоятельно переопределить метод-подсказку размера данных
    fn size_hint(&self) -> (usize, Option<usize>){
        (self.max, Some(self.max))
    }
}

fn test_custom_iterator(){
    let mut counter = Counter::new(5);

    assert_eq!(counter.next(), Some(1));
    assert_eq!(counter.next(), Some(2));
    assert_eq!(counter.next(), Some(3));
    assert_eq!(counter.next(), Some(4));
    assert_eq!(counter.next(), Some(5));
    assert_eq!(counter.next(), None);
}

fn test_unsugar(){
    // Мы можем итерироваться по вектору, так как он реализует трейт
    // IntoIterator c методом into_iter который и создает итератор
    let values = vec![1, 2, 3, 4, 5];
    for x in values {
        assert!(x > 0);
        // println!("{}", x);
    }

    // Таким образом код разворачивает в следующее
    let values = vec![1, 2, 3, 4, 5];
    {
        let result = match IntoIterator::into_iter(values) {
            // Создаем мутабельный итератор
            mut iter => loop {
                // Вызываем метод next пока выдаются значения
                let next;
                match iter.next() {
                    Some(val) => next = val,
                    None => break,
                };
                let x = next;
                let () = { 
                    assert!(x > 0);
                    //println!("{}", x); 
                };
            },
        };
        result
    }
}

fn test_iterator_gen_fn(){
    // Итератор является ленивым, он не делает ничего до тех пор, пока не вызовется next
    // Поэтому код ниже выдает предупреждение, так как у нас нету потребителя, который и вызывает в конечном счете next
    //let v = vec![1, 2, 3, 4, 5];
    //v.iter().map(|x| println!("{}", x));

    // Можно создавать бесконечный итератор и ограничивать методом take
    let numbers = 0..;
    let five_numbers = numbers.take(5);
    for number in five_numbers {
        assert!(number > -1 && number < 5);
        // println!("{}", number);
    }

    // Но надо помнить, что некторые методы итератора могут работать только с конечными итераторами, без take никак
    let ones = std::iter::repeat(1).take(10);
    let least = ones.min().unwrap(); // Oh no! An infinite loop!
    assert_eq!(least, 1);

    // Можно создавать итератор, который работает с переданным замыканием, которое генерирует данные
    let mut count = 0;
    let counter = std::iter::from_fn(move || {
        count += 1;
        if count < 6 {
            Some(count)
        } else {
            None
        }
    });
    let result: Vec<i32> = counter.collect::<Vec<i32>>(); // Collect позволяет создавать контейнер для типов, которые реализуют метод from_iter
    assert_eq!(result, &[1, 2, 3, 4, 5]); // Можно сравнивать с чем-то, что реализует трейт PartialEq

    // Можно просто итерироваться по повторяющемуся значению
    let mut four_fours = std::iter::repeat(4).take(4);
    assert_eq!(Some(4), four_fours.next());
    assert_eq!(Some(4), four_fours.next());
    assert_eq!(Some(4), four_fours.next());
    assert_eq!(Some(4), four_fours.next());
    assert_eq!(None, four_fours.next());

    // Можно создать итератор, который будет повторно вызывать лямбду
    // Разница с from_fn() в том, что здесь мы возвращаем конкретное значение, а не Option
    // а количество итераций регулируем с помощью take()
    let mut curr = 1;
    let mut pow2 = std::iter::repeat_with(move || { 
        let tmp = curr; 
        curr *= 2; 
        tmp 
    }).take(4);
    assert_eq!(Some(1), pow2.next());
    assert_eq!(Some(2), pow2.next());
    assert_eq!(Some(4), pow2.next());
    assert_eq!(Some(8), pow2.next());
    assert_eq!(None, pow2.next());

    // Можно создать итератор, который использует предыдущее значение итерации
    let powers_of_10 = std::iter::successors(Some(1_u16), |n| {
        n.checked_mul(10)
    });
    let result = powers_of_10.collect::<Vec<_>>();
    assert_eq!(result, &[1, 10, 100, 1_000, 10_000]);
}

fn test_iter_methods(){
    {
        // Получаем минимальное количество итераций и возможное максимальное
        // Если верхняя граница неизвестна - значит None
        let a = [1, 2, 3];
        let iter = a.iter();
        assert_eq!((3, Some(3)), iter.size_hint());

        // Фильтруем значения по четным
        let iter = (0..10).filter(|x| {
            x % 2 == 0
        });
        // Мы можем итерироваться от 0 до 10 раз, так как фильтрация может обросить все значения,
        // или все значения принять
        assert_eq!((0, Some(10)), iter.size_hint());

        // Мы можем добавить еще одну цепочку итераторов, которые точно имеют размер
        let iter = (0..10).filter(|x| {
            x % 2 == 0
        }).chain(15..20);
        // Тогда просто минимальные границы изменятся на 5
        assert_eq!((5, Some(15)), iter.size_hint());

        // У бесконечных итераторов нету верхней границы, а минимальное количество - максимально
        let iter = 0..;
        assert_eq!((usize::max_value(), None), iter.size_hint());
    }

    {
        // Просто получает значения из итератора и возвращает количество итераций
        let a = [1, 2, 3];
        assert_eq!(a.iter().count(), 3);
        let a = [1, 2, 3, 4, 5];
        assert_eq!(a.iter().count(), 5);
    }

    {
        // Получаем значения из итератора и возвращаем значение последнего или None если пустой итератор
        let a = [1, 2, 3];
        assert_eq!(a.iter().last(), Some(&3));
        let a = [1, 2, 3, 4, 5];
        assert_eq!(a.iter().last(), Some(&5));
    }

    {
        // Получаем энный элемент начиная с текущего положения итератора
        // Если значений нету - значит None
        let a = [1, 2, 3];
        let mut iter = a.iter();
        assert_eq!(iter.nth(1), Some(&2));
        assert_eq!(iter.nth(1), None);        
    }
    
    {
        // Возвращаем значение элемента с шагом n, пропуская элементы
        let a = [0, 1, 2, 3, 4, 5];
        let mut iter = a.iter().step_by(2);
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);        
    }
    
    {
        // Можно соединять итераторы в последовательную цепочку
        let a1 = [1, 2, 3];
        let a2 = [4, 5, 6];
        let mut iter = a1.iter().chain(a2.iter());
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), Some(&6));
        assert_eq!(iter.next(), None);
    }

    {
        // Метод zip объединяет 2 итератора в итератор с парами компонентов
        let a1 = [1, 2, 3];
        let a2 = [4, 5, 6];
        let mut iter = a1.iter().zip(a2.iter());
        assert_eq!(iter.next(), Some((&1, &4)));
        assert_eq!(iter.next(), Some((&2, &5)));
        assert_eq!(iter.next(), Some((&3, &6)));
        assert_eq!(iter.next(), None);
    }

    {
        // Можем конвертировать значения в новые, причем - даже нового типа
        let a = [1, 2, 3];
        let mut iter = a.iter().map(|x| {
            2 * x
        });
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), None);
    }

    {
        use std::sync::mpsc::channel;

        // Создаем канал
        let (tx, rx) = channel();
        // Итерируемся
        (0..5)
            .map(|x| {
                // Вычисляем новое значение
                x * 2 + 1
            })
            .for_each(move |x| {
                // Отправляем это новое значение в канал
                tx.send(x).unwrap()
            });
        let v: Vec<_> =  rx.iter().collect();
        assert_eq!(v, vec![1, 3, 5, 7, 9]);

        (0..5)
            // flat_map нужен для создания нового итератора из параметра
            // последующее итерирование уже будет по этим самым значениям
            .flat_map(|x| {
                let multiple_iterator = (x * 100)..(x * 110);
                return multiple_iterator;
            })
            // Объениняем значение и номер этого значения
            .enumerate()
            // Фильтруем значения, где сумма индекса и числа кратна трем
            .filter(|&(i, x)| {
                (i + x) % 3 == 0
            })
            // Выводим индекс и значение
            .for_each(|(_i, _x)| {
                //println!("{}:{}", i, x)
            });
    }

    {
        // Можно конвертировать только валидные значения с помощью фильтрации
        let a = ["1", "lol", "3", "NaN", "5"];
        let mut iter = a.iter().filter_map(|s| {
            let result : Option<i32> = s.parse().ok();
            result
        });
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), None);
    }

    {
        // Мы можем подсмотреть следующее значение, которое выдаст итератор
        // Для этого итератор должен реализовать трейт
        let xs = [1, 2, 3];
        let mut iter = xs.iter().peekable();
        assert_eq!(iter.peek(), Some(&&1)); // При этом возвращается ссылка на очередной элемент, а не непосредственно он сам
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.peek(), Some(&&3));
        assert_eq!(iter.peek(), Some(&&3)); // Причем, мы можем делать этот вызов несколько раз без проблем
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.peek(), None);      // Если итератор завершился - значит посмотреть тоже не можем
        assert_eq!(iter.next(), None);
    }

    {
        // Scan занимается тем, что хранит состояние внутри, это состояние можно модифицировать и выдавать значение какое-то
        let a = [1, 2, 3];
        let mut iter = a.iter()
            .scan(1, |state, &x| {
                // Каждую итерацию мы умножаем внутреннее состояние на элемент
                *state = *state * x;
                // Затем выдаем на выход отрицательное значение состояния
                Some(-*state)
            });
        
        assert_eq!(iter.next(), Some(-1));
        assert_eq!(iter.next(), Some(-2));
        assert_eq!(iter.next(), Some(-6));
        assert_eq!(iter.next(), None);        
    }

    {
        // Можно создавать новые под-итераторы по которым и будет происходить итерирование
        let words = ["alpha", "beta", "gamma"];
        let merged: String = words.iter()
                                  .flat_map(|s| {
                                      s.chars()
                                   })
                                  .collect();
        assert_eq!(merged, "alphabetagamma");        
    }

    {
        // flatten аналогичным образом просто разворачивает итератор с итераторами
        let words = ["alpha", "beta", "gamma"];
        let merged: String = words.iter()
                                .map(|s| s.chars())
                                .flatten()
                                .collect();
        assert_eq!(merged, "alphabetagamma");

        let data = vec![vec![1, 2, 3, 4], vec![5, 6]];
        let flattened = data.into_iter().flatten().collect::<Vec<u8>>();
        assert_eq!(flattened, &[1, 2, 3, 4, 5, 6]);
    }
    
    {
        // Создаем итератор, который будет случайным образом выдавать None/Some
        struct Alternate {
            state: i32,
        }
        impl Iterator for Alternate {
            type Item = i32;
            fn next(&mut self) -> Option<i32> {
                // Увеличиваем внутреннее значение на 1
                let val = self.state;
                self.state = self.state + 1;
                // Если четное значение, выдаем Some, если нет - None
                if val % 2 == 0 {
                    Some(val)
                } else {
                    None
                }
            }
        }
        let mut iter = Alternate { state: 0 };
        // В обычном состоянии выдает значения
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);
        // Но можно создать итератор-предохранитель, который будет выдавать None всегда после первого None
        let mut iter: std::iter::Fuse<_> = iter.fuse();
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);        
    }

    {
        let a = [1, 4, 2, 3];
        
        // Создаем сложный итератор по компонентам
        let sum = a.iter()
            // Каждый элемент будет клонироваться
            .cloned()
            // Фильтруем четные
            .filter(|x| {
                let is_even = x % 2 == 0;
                is_even
            })
            // Данный метод нужен, чтобы сложить все переданные значения в одно
            // Первый параметр - начальное значение
            // Второй - замыкание, которое принимает последнюю сумму + текущее значение
            .fold(0, |sum, i| {
                sum + i
            });
        
        assert_eq!(sum, 6);
        
        // Мы можем отследить значения для отладки
        let sum = a.iter()
            .cloned()
            .inspect(|_x| {
                //println!("Before filter: {}", _x);
            })
            .filter(|x| {
                x % 2 == 0
            })
            .inspect(|_x| {
                //println!("After filter: {}", _x)
            })
            .fold(0, |sum, i| {
                sum + i
            });
        
        assert_eq!(sum, 6); 
    }

    {
        let a = [1, 2, 3];
        let iter = a.iter();
        let sum: i32 = iter.take(2).fold(0, |acc, i| {
            acc + i
        });
        assert_eq!(sum, 3);
        // Если мы попытаемся использовать итератор снова, это не будет работать,
        // строка ниже выдаст ошибку, что мы используем уже перемещенный итератор
        // assert_eq!(iter.next(), None);
        
        // Но если мы будем использовать итератор по ссылке, то мы можем использовать этот итератор снова потом
        let a = [1, 2, 3];        
        let mut iter = a.iter();
        let sum: i32 = iter.by_ref().take(2).fold(0, |acc, i| {
            acc + i
        });        
        assert_eq!(sum, 3);
        assert_eq!(iter.next(), Some(&3)); // Можем спокойно использовать итератор дальше, так как мы работали с ссылкой
        assert_eq!(iter.next(), None);        
    }

    {
        // Мы можем собрать наш итератор в конкретную коллекцию
        // Но обязательно нужно указывать конкретный тип коллекции
        // Надо обязательно, чтобы коллекция имела трейт FromIter
        let a = [1, 2, 3];
        let doubled: Vec<i32> = a.iter()
                                 .map(|&x| x * 2)
                                 .collect();
        assert_eq!(vec![2, 4, 6], doubled);

        // Можно сконвертировать в map
        use std::collections::HashMap;
        let a = [1, 2, 3];
        let doubled: HashMap<usize, i32> = a.iter()
                                 .map(|&x| x * 2)
                                 .enumerate()
                                 .collect();
        let test_hash_map: HashMap<usize, i32> = map!{
            0 => 2,
            1 => 4,
            2 => 6
        };
        assert_eq!(test_hash_map, doubled);

        // Можно создавать вектор
        use std::collections::VecDeque;
        let a = [1, 2, 3];
        let doubled: VecDeque<i32> = a.iter().map(|&x| x * 2).collect();
        assert_eq!(2, doubled[0]);
        assert_eq!(4, doubled[1]);
        assert_eq!(6, doubled[2]);

        // Можно использовать turbofish синтаксис
        let a = [1, 2, 3];
        let doubled = a.iter().map(|x| x * 2).collect::<Vec<i32>>();
        assert_eq!(vec![2, 4, 6], doubled);

        // Можно выполнять конвертации массива в строки
        let chars = ['g', 'd', 'k', 'k', 'n'];
        let hello: String = chars.iter()
            .map(|&x| x as u8)
            .map(|x| (x + 1) as char)
            .collect();
        assert_eq!("hello", hello);

        // Если при вызове collect попадается ошибка, то вся сборка тоже заканчивается ошибкой
        let results = [Ok(1), Err("nope"), Ok(3), Err("bad")];
        let result: Result<Vec<_>, &str> = results.iter().cloned().collect();
        assert_eq!(Err("nope"), result);
        let results = [Ok(1), Ok(3)];
        let result: Result<Vec<_>, &str> = results.iter().cloned().collect();
        assert_eq!(Ok(vec![1, 3]), result); // gives us the list of answers
    }

    {
        // Можем создавать из итератора 2 коллекции по определенному признаку
        let a = [1, 2, 3];
        let (even, odd): (Vec<i32>, Vec<i32>) = a
            .iter()
            .partition(|&n| n % 2 == 0);
        assert_eq!(even, vec![2]);
        assert_eq!(odd, vec![1, 3]);
    }

    {
        // the checked sum of all of the elements of the array
        // Суммиуем все значения и возвращаем результат только если все было ок и каждое значение
        // оказалось валидным
        let a = [1, 2, 3];
        let initial_val = 0_i8;
        let sum = a.iter().try_fold(initial_val, |acc, &x| {
            // Если происходит переполнение, то None
            let sum: Option<i8> = acc.checked_add(x);
            sum
        });
        assert_eq!(sum, Some(6));


        let a = [10, 20, 30, 100, 40, 50];
        let mut it = a.iter();
        let sum = it.try_fold(0i8, |acc, &x| {
            // Тут происходит переполнение
            acc.checked_add(x)
        });
        assert_eq!(sum, None);
        // Что интересно, при ошибке - оставшиеся элементы до переполнения остаются доступны у итератора
        assert_eq!(it.len(), 2);
        assert_eq!(it.next(), Some(&40));
    }
}

fn main() {
    // У коллекций есть несколько методов создания итераторов
    // iter() - создает итератор, который позволяет итерироваться по ссылкам &T.
    // iter_mut() - создает итератор, который позволяет итерироваться по мутабельным ссылкам &mut T.
    // into_iter() - создает итератор, который перемещает владение содержимого
    
    test_custom_iterator();
    test_unsugar();
    test_iterator_gen_fn();
    test_iter_methods();
}
