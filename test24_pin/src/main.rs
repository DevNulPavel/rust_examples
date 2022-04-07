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
            // Мы должны создавать указатель только тогда, когда данные размещены
            // иначе указатель уже сместится
            slice: NonNull::dangling(),
            // Данный фейковый тип говорит, что данная структура неперемещаемая в памяти
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

    fn new_boxed(data: String) -> Box<Self>{
        let res = Unmovable {
            data,
            slice: NonNull::dangling(),
            _pin: PhantomPinned, // Данный фейковый тип говорит, что данная структура неперемещаемая в памяти
        };
        let mut boxed: Box<Unmovable> = Box::new(res);

        // Создаем слайс на строку из данных
        let slice = NonNull::from(&boxed.data);
        // Мы знаем, что это безопасно, так как модификация поля не изменяет расположение структуры в памяти
        boxed.as_mut().slice = slice;

        boxed
    } 
}

fn test_pin_data() {
    {
        // Создаем данные
        let unmoved: Pin<Box<Unmovable>> = Unmovable::new("hello".to_string());

        // Указатель должен указывать на правильное расположение,
        // так как структура не перемещалась.
        // В то же время, мы можем спокойно перемещать Pin
        let still_unmoved: Pin<Box<Unmovable>> = unmoved;
        assert_eq!(still_unmoved.slice, NonNull::from(&still_unmoved.data));

        // Так как наш тип не реализует Unpin, то это значит, что компиляция сфейлится
        // Таким образом Pin по большому счету запрещает вызовы swap
        // let mut new_unmoved = Unmovable::new("world".to_string());
        // std::mem::swap(&mut *still_unmoved, &mut *new_unmoved);
    }

    // Но вроде бы как обычный box без проблем решает проблему?
    // То есть единственный плюс Pin только для обычных ссылок?
    {
        let unmoved = Unmovable::new_boxed("test".to_owned());
        let still_unmoved = unmoved;
        assert_eq!(still_unmoved.slice, NonNull::from(&still_unmoved.data));
    }

    // Вроде бы как адрес данных внутри Box не изменяется никак
    // То есть Pin лишь запрещает вызовы swap?
    {
        let box1 = Box::new("Test data");
        let box1_data_address = box1.as_ptr() as usize;
        let box2 = box1;
        let box2_data_address = box2.as_ptr() as usize;
        assert_eq!(box1_data_address, box2_data_address);
    }
    {
        let pined_box1 =  Box::pin("Test data");
        let pined_box1_addr = pined_box1.as_ptr();
        let pined_box2 =  pined_box1;
        let pined_box2_addr = pined_box2.as_ptr();
        assert_eq!(pined_box1_addr, pined_box2_addr);
    }
}

fn main() {
    test_pin_data();
    println!("Done");
}
