#![warn(clippy::all)]
#![allow(dead_code)]

// Perform a forward FFT of size 1234
extern crate rustfft;

use std::sync::Arc;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_complex::Complex32;
use rustfft::num_traits::Zero;

fn print_data(prefix: &str, data: &[Complex32]){
    data
        .iter()
        .enumerate()
        .for_each(|(i, val)|{
            println!("{}-> {}: re {}, im {}", prefix, i, val.re, val.im);
        });
}

fn print_result(prefix: &str, data: &[Complex32]){
    data
        .iter()
        .take(data.len() / 2)
        .enumerate()
        .for_each(|(i, val)|{
            let amplitude = (1.0 / data.len() as f32) * (val.re * val.re + val.im * val.im).sqrt();
            //println!("{}-> {}: re {}, im {}, amp: {}", prefix, i, val.re, val.im, amplitude);
            
            // Нулевое значение - базовая составляющая, затем идут частоты (гармоники)
            println!("{}-> {}: amp: {}", prefix, i, amplitude);
        });
}

fn make_test_input()-> Vec<Complex32> {
    let input: Vec<Complex32> = vec![
        Complex32::new(1.0, 0.0),
        Complex32::new(0.5, 0.0),
        Complex32::new(0.2, 0.0),
        Complex32::new(0.1, 0.0),
        Complex32::new(0.0, 0.0),
        Complex32::new(0.0, 0.0),
        Complex32::new(0.0, 0.0),
        Complex32::new(0.0, 0.0)
    ];
    input
}

fn test_forward_transform_with_planner(){
    const DATA_SIZE: usize = 128;

    // Предаллоцируем вектор нужного размера с нулевыми значениями
    let mut i = 0 as f32;
    let mut input:  Vec<Complex32> = std::iter::repeat_with(||{ 
            // Синус обновляется целиком 2 раза, это значит, что в спектре будет 2я гармоника
            const HARMONIC_NUMBER1: f32 = 3.0;
            const HARMONIC_NUMBER2: f32 = 5.0;

            const HARMONIC_POWER_1: f32 = 0.7;
            const HARMONIC_POWER_2: f32 = 0.3;

            const STEP1: f32 = std::f32::consts::PI * 2.0 * HARMONIC_NUMBER1 / (DATA_SIZE as f32);
            const STEP2: f32 = std::f32::consts::PI * 2.0 * HARMONIC_NUMBER2 / (DATA_SIZE as f32);

            // Не забываем сделать нормализацию
            let result: Complex32 = Complex32::new(((STEP1 * i).sin() * HARMONIC_POWER_1 + 
                                                    (STEP2 * i).sin() * HARMONIC_POWER_2) / 2.0, 0.0);

            i += 1.0;

            result
        })
        .take(DATA_SIZE)
        .collect();
    let mut output: Vec<Complex32> = vec![Complex::zero(); DATA_SIZE];
    
    print_data("Input", &input);

    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    let mut planner = FFTplanner::new(false);

    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    let fft = planner.plan_fft(DATA_SIZE);

    // Обрабатываем данные
    // Входные данные мутабельные, так как они используются в качестве буффера
    // Как результат - там будет мусор после вычисления
    fft.process(&mut input, &mut output);
    
    // The fft instance returned by the planner is stored behind an `Arc`, so it's cheap to clone
    // Экземпляр FFT хранится в Arc умном указателе, можно легко его клонировать
    //let fft_clone = Arc::clone(&fft);

    println!("\n\n");
    print_result("Output", &output);
}

fn main() {
    test_forward_transform_with_planner();
}
