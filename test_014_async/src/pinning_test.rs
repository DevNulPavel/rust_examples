use std::future::Future;


// Функция, принимающая `Future`, которая реализует `Unpin`.
fn execute_unpin_future(x: impl Future<Output = ()> + Unpin) {
}

pub fn pining_test_example(){
    use pin_utils::pin_mut; // `pin_utils` - удобный пакет, доступный на crates.io

    //let fut = async { 
    //};
    //execute_unpin_future(fut); // Ошибка: `fut` не реализует типаж `Unpin`

    // Закрепление с помощью `Box`:
    let fut = async {
    };
    let fut = Box::pin(fut);
    execute_unpin_future(fut); // OK

    // Закрепление с помощью `pin_mut!`:
    let fut = async {
    };
    pin_mut!(fut);
    execute_unpin_future(fut); // OK
}