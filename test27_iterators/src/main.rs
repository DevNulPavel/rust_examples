

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
