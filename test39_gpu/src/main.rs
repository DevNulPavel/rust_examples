extern crate collenchyma as co;
// extern crate collenchyma_nn as nn;

use co::prelude::*;
// use nn::*;

/// Запись данных в буффер
fn write_to_memory<T: Copy>(output: &mut MemoryType, input: &[T]) {
    // Проверяем, что это нативный CPU буффер из выходного буффера
    if let &mut MemoryType::Native(ref mut mem) = output {
        // Буффер, в который записываем
        let mem_buffer = mem.as_mut_slice::<T>();
        
        // Итератор по входным данным
        let iterator = input
            .iter()
            .enumerate();

        // Записываем в выходной буффер
        for (index, datum) in iterator {
            mem_buffer[index] = *datum;
        }
    }
}

fn main() {
    // Инициализаруем OpenCL бэкенд
    let gpu_device = Backend::<OpenCL>::default().unwrap();
    // Создаем CPU бэкенд
    let cpu_device = Backend::<Native>::default().unwrap();

    // Инициализируем буфферы
    let mut x = SharedTensor::<f32>::new(gpu_device.device(), &(1, 1, 3)).unwrap();
    let mut result = SharedTensor::<f32>::new(gpu_device.device(), &(1, 1, 3)).unwrap();

    // Заполняем входной буффер входными данными в виде 1.0
    let payload: &[f32] = &std::iter::repeat(1.0_f32)
        .take(x.capacity())
        .collect::<Vec<f32>>();
    
    // Прицепляем CPU буффер к буфферу на GPU
    x.add_device(cpu_device.device()).unwrap();

    // Синхронизуем работу с CPU
    x.sync(cpu_device.device()).unwrap();

    // Записываем данные
    write_to_memory(x.get_mut(cpu_device.device()).unwrap(), payload); // Write to native host memory.
 
    // Синхронизуем GPU c CPU
    x.sync(gpu_device.device()).unwrap();
 
    // Запускаем сигмоид операцию, предоставляемую NN плагином для работы на GPU
    // gpu_device.sigmoid(&mut x, &mut result).unwrap();

    // Добавляем к результату CPU девайс
    result.add_device(cpu_device.device()).unwrap();

    // Синхронизуем работу
    result.sync(cpu_device.device()).unwrap();

    // Выдаем результат
    let result_data = result
        .get(cpu_device.device())
        .unwrap()
        .as_native()
        .unwrap()
        .as_slice::<f32>();
    println!("{:?}", result_data);
}