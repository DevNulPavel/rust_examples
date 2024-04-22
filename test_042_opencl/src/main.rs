extern crate ocl;

use ocl::ProQue;

fn trivial() -> ocl::Result<()> {
    // Исходный код нашего вычислителя, просто к большому буфферу добавляем скалярное значение
    let src = r#"
        __kernel void add(__global float* buffer, float scalar) {
            buffer[get_global_id(0)] += scalar;
        }
    "#;

    const SIZE: usize = 1024 * 1024 * 16; // 16 Mb

    // Создаем обработчик задач для нужной размерности
    let pro_que = ProQue::builder()
        .src(src)
        .dims(SIZE)
        .build()?;

    // Буффер с нулевыми значениями
    let buffer = pro_que
        .create_buffer::<f32>()?;

    // Получаем нашу вычислительную функцию
    // Выставляем наши аргументы
    let kernel = pro_que
        .kernel_builder("add")
        .arg(&buffer)
        .arg(14.0_f32)
        .build()?;

    unsafe { 
        kernel.enq()?; 
    }

    // Создаем буффер для результата
    let mut vec = vec![0.0f32; buffer.len()];

    // Вычитываем результат
    buffer.read(&mut vec).enq()?;

    println!("The value at index [{}] is now '{}'!", 200007, vec[200007]);
    Ok(())
}

fn main(){
    trivial().unwrap();
}