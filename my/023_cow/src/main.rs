use std::borrow::Cow;

// На вход принимаем умный указатель
fn abs_all(input: &mut Cow<[i32]>) {
    // Идем в цикле
    for i in 0..input.len() {
        // Получаем значение небезопасным способом
        let v = input[i];
        // Проверяем, если значение больше нуля - модифицируем массив
        if v < 0 {
            // Клонируем наш массив внутри умного указателя в новый мутабельный вектор
            let mutable_ref = input.to_mut();
            // И меняем наше значение
            mutable_ref[i] = -v;
        }
    }
}

fn test_simple_cow(){
    // Не происходит никакого клонирования, так как параметр не модифицируется, так как все значения положительные
    let slice: [i32; 3] = [0, 1, 2];
    let mut input: Cow<'_, [i32]> = Cow::from(&slice[..]);
    abs_all(&mut input);
    println!("Before: {:?}, After: {:?}", slice, input);
    // Видно, что указатели на данные равны
    assert_eq!(slice.as_ptr(), input.as_ptr());

    // Здесь же произойдет клонирование исходного массива для модификации внутри функции
    let slice: [i32; 3] = [-1, 0, 1];
    let mut input: Cow<'_, [i32]> = Cow::from(&slice[..]);
    abs_all(&mut input);
    println!("Before: {:?}, After: {:?}", slice, input);
    // Видно, что указатели на данные НЕ равны
    assert_ne!(slice.as_ptr(), input.as_ptr());
    
    // Здесь же не происходит никакого клонирования так как Cow уже и так владеет данными изначально
    let mut input = Cow::from(vec![-1, 0, 1]);
    abs_all(&mut input);
}

struct Items<'a, X: 'a> 
    where [X]: ToOwned<Owned = Vec<X>> 
{
    values: Cow<'a, [X]>,
}

impl<'a, X: Clone + 'a> Items<'a, X> 
    where [X]: ToOwned<Owned = Vec<X>> 
{
    fn new(v: Cow<'a, [X]>) -> Self {
        Items{ values: v }
    }
}

fn test_cow_in_struct(){
    // Создаем контейер, который заимствует данные из слайса
    let readonly: [i32; 2] = [1, 2];
    let data_slice: &[i32] = &readonly[..];
    // into метод позволяет как раз создавать такие Cow из слайсов
    let data_ref: Cow<'_, [i32]> = data_slice.into();

    // Так как Cow - это enum, то это значит, что мы можем сравнить с паттерном и работать с внутренним содержимым
    match data_ref {
        Cow::Borrowed(borrowed) => {
            println!("Borrowed {:?}", borrowed);
        },
        Cow::Owned(owned) => {
            panic!("Owned val {:?}", owned);
        },
    }

    // Создали контейнер
    let borrowed = Items::new(data_ref);

    // Так как Cow - это enum, то это значит, что мы можем сравнить с паттерном и работать с внутренним содержимым
    match borrowed {
        Items{ values: Cow::Borrowed(b) } => {
            println!("Borrowed {:?}", b);
        },
        Items{ values: Cow::Owned(b) } => {
            panic!("Owned val {:?}", b);
        },
    }

    // Создаем копию нашего указателя
    let mut clone_on_write = borrowed;
    // Изменяем наши данные, тем самым - создавая копию
    clone_on_write.values.to_mut().push(3);
    println!("Сlone_on_write = {:?}", clone_on_write.values);

    // The data was mutated. Let check it out.
    match clone_on_write {
        Items{ values: Cow::Owned(_) } => {
            println!("Clone_on_write contains owned data");
        },
        _ => {
            panic!("expect owned data");
        },
    }
}

fn main() {
    test_simple_cow();
    test_cow_in_struct();
}

