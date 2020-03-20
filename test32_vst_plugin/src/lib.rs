#![allow(unused_imports)]
#![allow(dead_code)]

mod parameters;

use std::sync::Arc;
use rand::random;
use vst::plugin_main;
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::api::Events;
use vst::plugin::{
    Info,
    Plugin,
    PluginParameters,
    Category
};
use rustfft::FFTplanner;
use rustfft::num_traits::Zero;
use rustfft::num_complex::{
    Complex,
    Complex32
};

use parameters::SimplePluginParameters;

struct BasicPlugin{
    total_notes: i32,
    last_input_buffer: Vec<Vec<f32>>,
    params: Arc<SimplePluginParameters>
}

impl Default for BasicPlugin {
    fn default() -> BasicPlugin {
        BasicPlugin {
            total_notes: 0,
            last_input_buffer: vec![vec![], vec![]],
            params: Arc::new(SimplePluginParameters::default())
        }
    }
}

impl Plugin for BasicPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "DevNul's FFT Plugin".to_string(),
            vendor: "DevNul".to_string(),
            unique_id: 1357312,         // Уникальный номер, чтобы отличать плагины
            presets: 0,                 // Количество пресетов
            inputs: 2,                  // Сколько каналов звука на входе
            outputs: 2,                 // Каналы звука на выходе
            version: 0001,              // Версия плагина 
            category: Category::Effect, // Тип плагина
            parameters: 2,
            //initial_delay, 
            //preset_chunks, 
            //f64_precision, 
            //silent_when_stopped

            ..Default::default()
        }
    }

    // Выдаем ссылку на шареный объект параметров
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.params.clone()
    }

    // Функция, которая вызывается на события, такие как MIDI и тд
    /*fn process_events(&mut self, events: &Events) {
        // Идем по всем ивентам, некоторые из них - MIDI
        // Обрабатывать будем только MIDI 
        for event in events.events() {
            match event {
                Event::Midi(ev) => {

                    // Проверяем, что нота нажата или нет
                    // Первый байт события говорит нам, что нота нажата или нет
                    // Высота ноты хранится во втором байте 
                    // https://www.midi.org/specifications/item/table-1-summary-of-midi-message
                    match ev.data[0] {
                        // Если нота нажата
                        // 0b10010000
                        144 => {
                            self.total_notes += 1
                        },
                        // Если нота не нажата
                        // 0b10000000
                        128 => {
                            self.total_notes -= 1
                        },
                        _ => (),
                    }
                },
                // We don't care if we get any other type of event
                _ => (),
            }
        }
    }*/

    fn process(&mut self, buffer: &mut AudioBuffer<f32>){
        let threshold = self.params.get_threshold();
        let volume = self.params.get_volume();
    
        // https://habr.com/ru/post/430536/

        // Создаем итератор по парам элементов, входа и выхода
        let mut i = 0;
        for (input, output) in buffer.zip() {
            if self.last_input_buffer[i].len() < input.len() {
                self.last_input_buffer[i].resize(input.len(), 0.0_f32);
            }
    
            let last_in_buf: &mut Vec<f32> = &mut self.last_input_buffer[i];
            
            handle_data(input, output, last_in_buf, threshold, volume);
    
            i += 1;
        }
    } 
}

pub fn handle_data( input: &[f32], 
                output: &mut [f32], 
                last_in_buf: &mut Vec<f32>, 
                threshold: f32,
                volume: f32){

    const BUFFER_MUL: usize = 1;

    //const INTERSECTION = 0.75;

    let window_size = input.len() * BUFFER_MUL;

    // let wind = 0.5 * (1.0 - ((2.0_f32 * std::f32::consts::PI * i as f32) / (window_size as f32 - 1.0_f32)).cos());
    let window = apodize::hanning_iter(window_size).collect::<Vec<f64>>();

    // Первая секция
    let mut input_fft_1: Vec<Complex32> = last_in_buf
        .iter()
        .flat_map(|val|{
            std::iter::repeat(val).take(BUFFER_MUL)
        })         
        .zip(window.iter().map(|val| *val as f32 ))
        .map(|(val, wind)|{
            //println!("2 {} * {} = {:?}", wind, *val, *val * wind);
            Complex32::new(*val * wind, 0.0)
            // Complex32::new(*val, 0.0)
        })
        .collect();

    //println!("");        

    let mut output_fft_1: Vec<Complex32> = vec![Complex::zero(); window_size];

    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    // Обрабатываем данные
    // Входные данные мутабельные, так как они используются в качестве буффера
    // Как результат - там будет мусор после вычисления
    FFTplanner::new(false)
        .plan_fft(output_fft_1.len())
        .process(&mut input_fft_1, &mut output_fft_1);

    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    FFTplanner::new(true)
        .plan_fft(output_fft_1.len())
        .process(&mut output_fft_1, &mut input_fft_1);

    let inv_len = 1.0 / (input_fft_1.len() as f32);
    input_fft_1
        .iter_mut()
        .for_each(|val|{
            *val *= inv_len;
            // println!("1 {:?}", *val);
        });

    //println!("");

    // let mut window_res = apodize::hanning_iter(input_fft_1.len()).collect::<Vec<f64>>();
    // input_fft_1
    //     .iter_mut()
    //     .zip(window_res.iter_mut())
    //     .for_each(|(val, wind)|{
    //         *val *= *wind as f32;
    //     });

    //let window = apodize::hanning_iter(window_size).collect::<Vec<f64>>();                
    //let window = sample::window::hanning::<f32>(window_size);

    // Вторая секция
    let mut input_fft_2: Vec<Complex32> = last_in_buf
        .iter()
        .skip(last_in_buf.len() / 2) // /4
        .take(last_in_buf.len() / 2) // * 3 / 4
        .chain(input
            .iter()
            .take(input.len() / 2))
        .flat_map(|val|{
            std::iter::repeat(val).take(BUFFER_MUL)
        })
        // .map(|val|{
        //     Complex32::new(*val, 0.0)
        // })                
        // .zip(window.iter().map(|val| *val as f32 ))
        .enumerate()
        .map(|(i, val)|{
            let wind = 0.5 * (1.0 - ((2.0_f32 * std::f32::consts::PI * i as f32) / (window_size as f32 - 1.0_f32)).cos());
            //println!("2 {} * {} = {:?}", wind, *val, *val * wind);
            // Complex32::new(*val, 0.0)
            Complex32::new(*val * wind, 0.0)
        })
        .collect();

    let mut output_fft_2: Vec<Complex32> = vec![Complex::zero(); window_size];
    
    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    FFTplanner::new(false)
        .plan_fft(input_fft_2.len())
        .process(&mut input_fft_2, &mut output_fft_2);

    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    FFTplanner::new(true)
        .plan_fft(output_fft_2.len())
        .process(&mut output_fft_2, &mut input_fft_2);

    let inv_len = 1.0 / (input_fft_2.len() as f32);
    input_fft_2
        .iter_mut()
        .for_each(|val|{
            *val *= inv_len;
            //println!("2 {:?}", *val);
        });

    // let mut window_res = apodize::hanning_iter(input_fft_2.len()).collect::<Vec<f64>>();
    // input_fft_2
    //     .iter_mut()
    //     .zip(window_res.iter_mut())
    //     .for_each(|(val, wind)|{
    //         *val *= *wind as f32;
    //     });

    //println!("");

    // Третья секция
    let mut input_fft_3: Vec<Complex32> = input
        .iter()
        .flat_map(|val|{
            std::iter::repeat(val).take(BUFFER_MUL)
        })
        // .map(|val|{
        //     Complex32::new(*val, 0.0)
        // })                
        // .zip(window.iter().map(|val| *val as f32 ))
        .enumerate()
        .map(|(i, val)|{
            let wind = 0.5 * (1.0 - ((2.0_f32 * std::f32::consts::PI * i as f32) / (window_size as f32 - 1.0_f32)).cos());
            //println!("3 {} * {} = {:?}", wind, *val, *val * wind);
            // Complex32::new(*val, 0.0)
            Complex32::new(*val * wind, 0.0)
        })
        .collect();

    let mut output_fft_3: Vec<Complex32> = vec![Complex::zero(); window_size];

    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    FFTplanner::new(false)
        .plan_fft(input_fft_3.len())
        .process(&mut input_fft_3, &mut output_fft_3);

    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    FFTplanner::new(true)
    .plan_fft(output_fft_3.len())
    .process(&mut output_fft_3, &mut input_fft_3);

    let inv_len = 1.0 / (input_fft_3.len() as f32);
        input_fft_3
        .iter_mut()
        .for_each(|val|{
            *val *= inv_len;
            //println!("2 {:?}", *val);
        });


    // Сохраняем текущие данные из нового буффера в старый
    last_in_buf.copy_from_slice(input);

    let iter = input_fft_1
        .into_iter()
        .skip(output.len() / 2)
        .take(output.len() / 2)
        .zip(input_fft_2
            .split_at(output.len() / 2)
            .0
            .iter()
            .take(output.len() / 2))
        .step_by(BUFFER_MUL)
        // .zip(window_res.iter().map(|val| *val as f32))
        // .map(|((val1, val2), wind)|{
        //     (val1.re + val2.re) * (1.0 - wind)
        // })
        .map(|(val1, val2)|{
            // let val = (val1.re + val2.re) * 2.0 / 3.0;
            // let val = (val1.re + val2.re) / 2.0;
            let val = val1.re + val2.re;
            //println!("{} + {} = {}", val1, val2, val);
            val
            //val2.re
            // val1.re
        }) 
        .chain(input_fft_2
            .split_at(output.len() / 2)
            .1
            .into_iter()
            .zip(input_fft_3
                .into_iter()
                .take(output.len() / 2))
            .map(|(val1, val2)|{
                let val = val1.re + val2.re;
                //println!("{} + {} = {}", val1, val2, val);
                val
            }))
        .zip(output.into_iter());

    for (in_sample, out_sample) in iter {    
        let val = in_sample;

        *out_sample = val;

        // Эмулируем клиппинг значений
        *out_sample = if val > threshold {
            threshold
        } else if val < -threshold {
            -threshold
        } else {
            val
        };

        *out_sample *= volume;
    }
}

plugin_main!(BasicPlugin); // Important!

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_process(){
        
    }
}