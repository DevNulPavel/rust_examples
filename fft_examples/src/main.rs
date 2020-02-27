#![warn(clippy::all)]
#![allow(dead_code)]

// Perform a forward FFT of size 1234
extern crate rustfft;
extern crate plotters;

//use std::sync::Arc;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use rustfft::num_complex::Complex32;
use rustfft::num_traits::Zero;
use plotters::prelude::*;

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

fn plot_results(input: &[Complex32], out: &[Complex32]) -> Result<(), Box<dyn std::error::Error>> {
    let data_size = out.len() / 2;

    let in_iter = input
        .iter()
        .enumerate()
        .map(|(i, val)|{
            (i as f32, val.re)
        });
    
    let out_iter = out
        .iter()
        .take(data_size)
        .enumerate()
        .map(|(i, val)|{
            let res = (1.0 / out.len() as f32) * (val.re * val.re + val.im * val.im).sqrt();
            (i as f32, res)
        });


    let root = BitMapBackend::new("0.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(0.0f32..data_size as f32, -1.0f32..1.0f32)?;

    chart.configure_mesh().draw()?;

    //let test_iter = (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x));

    chart
        .draw_series(LineSeries::new(
            in_iter,
            &BLUE,
        ))?
        .label("Input signal")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart
        .draw_series(LineSeries::new(
            out_iter,
            &RED,
        ))?
        .label("Freq amplitudes")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
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

fn make_harm(number: u32, power: f32, i: usize, data_size: usize) -> Complex32 {
    let step: f32 = std::f32::consts::PI * 2.0 * number as f32 / (data_size as f32);
    let result: Complex32 = Complex32::new((step * i as f32).sin() * power, 0.0);
    result
}

fn test_forward_transform_with_planner(){
    const DATA_SIZE: usize = 256;

    // Предаллоцируем вектор нужного размера с нулевыми значениями
    let mut input:  Vec<Complex32> = std::iter::repeat(0.0f32)
        .enumerate()
        .map(|(i, val)|{
            (i, val, 0.0f32)
        })
        .map(|(i, val, step)|{
            let result: Complex32 = make_harm(4, 0.4, i, DATA_SIZE);
            (i, val + result, step + 1.0f32)
        })
        .map(|(i, val, step)|{
            let result: Complex32 = make_harm(13, 0.8, i, DATA_SIZE);
            (i, val + result, step + 1.0f32)
        })
        .map(|(i, val, step)|{
            let result: Complex32 = make_harm(27, 0.4, i, DATA_SIZE);
            (i, val + result, step + 1.0f32)
        })
        .map(|(i, val, step)|{
            let result: Complex32 = make_harm(40, 0.8, i, DATA_SIZE);
            (i, val + result, step + 1.0f32)
        })
        .map(|(_, val, step)|{
            // Не забываем сделать нормализацию
            Complex32::new(val.re / step, 0.0)
        })
        .take(DATA_SIZE)
        .collect();
    let mut output: Vec<Complex32> = vec![Complex::zero(); DATA_SIZE];
    
    //print_data("Input", &input);

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
    //print_result("Output", &output);

    plot_results(&input, &output).ok();
}

fn main() {
    test_forward_transform_with_planner();
}
