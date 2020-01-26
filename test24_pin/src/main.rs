use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;

// Это ссылающая на себя структура данных, так как слайс ссылается на поле данных
// Мы не можем сказать компилятору, что это нормальная ссылка,
// так как данный паттерно не может быть объяснен стандартными правилами заимствования.
// Вместо этого мы используем сырой указатель, который точно не будет нулевым,
// так как он указывает на строку.
struct Unmovable {
    data: String,
    slice: NonNull<String>,
    _pin: PhantomPinned,
}

impl Unmovable {
    // Чтобы обеспечить, что данные не перемещаются когда функция завершается,
    // мы размещаем данные в куче, где они будут оставаться весь лайфтайм объекта.
    // И доступны они будут толкьо через указатель на данные.
    fn new(data: String) -> Pin<Box<Self>> {
        let res = Unmovable {
            data,
            // Cоздаем указатель с висячей ссылкой, который будет установлен ниже
            // TODO: ???
            // Мы должны создавать указатель только тогда, когда данные размещены
            // иначе указатель уже сместится
            // we only create the pointer once the data is in place
            // otherwise it will have already moved before we even started
            slice: NonNull::dangling(),
            _pin: PhantomPinned,
        };
        let mut boxed: Pin<Box<Unmovable>> = Box::pin(res);

        // Создаем слайс на строку из данных
        let slice = NonNull::from(&boxed.data);
        // Мы знаем, что это безопасно, так как модификация поля не изменяет расположение структуры в памяти
        unsafe {
            // Получаем ссылку
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            // Сохраняем слайс
            Pin::get_unchecked_mut(mut_ref).slice = slice;
        }
        boxed
    }
}

fn test_pin_data() {
    // Создаем данные
    let unmoved: Pin<Box<Unmovable>> = Unmovable::new("hello".to_string());
    // Указатель должен указывать на правильное расположение,
    // так как структура не перемещалась.
    // В то же время, мы можем спокойно перемещать указатель
    let still_unmoved: Pin<Box<Unmovable>> = unmoved;
    assert_eq!(still_unmoved.slice, NonNull::from(&still_unmoved.data));

    // Since our type doesn't implement Unpin, this will fail to compile:
    // let mut new_unmoved = Unmovable::new("world".to_string());
    // std::mem::swap(&mut *still_unmoved, &mut *new_unmoved);
}

fn main() {
    test_pin_data();
}
