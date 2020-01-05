
// Говорим, что хотим использовать соседние модули
mod cpupool_test;
mod futures_example;

// Описываем используемые модули из файликов
//use crate::executor_test::text_executor_example;

fn main() {
    cpupool_test::test_tasks();
    // futures_example::test_futures();
}