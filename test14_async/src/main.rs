
// Говорим, что хотим использовать соседние модули
mod executor_test;
mod async_await_test;
mod folder_test;

// Описываем используемые модули из файликов
use crate::executor_test::text_executor_example;

fn main() {
    text_executor_example();
    
    folder_test::test_func_1();
    folder_test::test_func_2();
}