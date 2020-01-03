
// Говорим, что хотим использовать соседние модули
mod cpupool_test;

// Описываем используемые модули из файликов
//use crate::executor_test::text_executor_example;

fn main() {
    cpupool_test::test_tasks();
}