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


pub struct BasicPlugin{
    total_notes: i32,

    sample_rate: f32,

    previous_input: Vec<Vec<f32>>,
    previous_result: Vec<Vec<Complex32>>,

    buffer_fft: Vec<Complex32>,

    fft_1: Vec<Complex32>,
    fft_2: Vec<Complex32>,
    fft_3: Vec<Complex32>,
    
    params: Arc<SimplePluginParameters>
}

impl Default for BasicPlugin {
    fn default() -> BasicPlugin {
        BasicPlugin {
            total_notes: 0,
            sample_rate: 44100.0,
            previous_result: vec![vec![], vec![]],
            previous_input: vec![vec![], vec![]],
            buffer_fft: vec![],
            fft_1: vec![],
            fft_2: vec![],
            fft_3: vec![],
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
            parameters: 3,
            //initial_delay, 
            //preset_chunks, 
            //f64_precision, 
            //silent_when_stopped

            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
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

    fn start_process(&mut self) {
    }

    fn stop_process(&mut self) {
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>){
        let threshold = self.params.get_threshold();
        let volume = self.params.get_volume();
        let freq = self.params.freq.get();
    
        // https://habr.com/ru/post/430536/

        // Создаем итератор по парам элементов, входа и выхода
        let mut i = 0;
        for (input, output) in buffer.zip() {
            if input.len() == 0 || output.len() == 0{
                break;
            }

            self.handle_data(i, input, output, freq, threshold, volume);
            i += 1;
        }
    } 
}

impl BasicPlugin{
    pub fn handle_data(&mut self, 
                    channel: usize,
                    input: &[f32], 
                    output: &mut [f32], 
                    freq: f32,
                    threshold: f32,
                    volume: f32){

        // https://habr.com/ru/post/430536/
        // https://habr.com/ru/post/196374/

        const BUFFER_MUL: usize = 1;

        // Результат прошлых вычислений
        let previous_result: &mut Vec<Complex32> = &mut self.previous_result[channel];
        check_buffer_size(previous_result, input.len() * BUFFER_MUL);

        // Прошлый вход
        let previous_input: &mut Vec<f32> = &mut self.previous_input[channel];
        check_buffer_size(previous_input, input.len() * BUFFER_MUL);

        check_buffer_size(&mut self.buffer_fft, input.len() * BUFFER_MUL);

        // Первая секция - используем прошлый результат с окном
        let result_fft_1 = &*previous_result;

        // Вторая секция - перекрывающийся прошлый вход и новый
        let result_fft_2 = {
            // Увеличиваем размер буфферов если надо
            check_buffer_size(&mut self.fft_2, input.len() * BUFFER_MUL);

            // Итератор по нужным данным
            let input_fft = get_section_iterator_2(previous_input, input);

            // Обновляем первый FFT буффер новыми значениями из итератора
            update_with_iter(&mut self.fft_2, input_fft);

            fft_process(self.sample_rate, freq, &mut self.fft_2, &mut self.buffer_fft);

            &self.fft_2
        };

        // Третья секция - текущий вход
        let result_fft_3 = {
            // Увеличиваем размер буфферов если надо
            check_buffer_size(&mut self.fft_3, input.len() * BUFFER_MUL);

            // Итератор по нужным данным
            let input_fft = get_section_iterator_3(input);

            // Обновляем первый FFT буффер новыми значениями из итератора
            update_with_iter(&mut self.fft_3, input_fft);

            fft_process(self.sample_rate, freq, &mut self.fft_3, &mut self.buffer_fft);

            &self.fft_3        
        };
        
        // Итератор по результатам
        let iter = crossfade_results(result_fft_1, result_fft_2, result_fft_3, output);

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

        // Сохраняем текущие данные из нового буффера в старый
        previous_result.copy_from_slice(result_fft_3);
        // Сохраняем прошлый вход
        previous_input.copy_from_slice(input);
    }
}

fn fft_process(sample_rate: f32, filter_freq: f32, input_fft_1: &mut [Complex32], buffer_fft: &mut [Complex32]){
    //const SAMPLE_RATE: f32 = 48_000.0;
    //const SAMPLE_DURATION: f32 = 1.0 / SAMPLE_RATE * 1000.0; // Длительность семпла в mSec

    let input_len = input_fft_1.len() as f32; 
    let fft_sample_hz: f32 = sample_rate / input_len;

    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    // Обрабатываем данные
    // Входные данные мутабельные, так как они используются в качестве буффера
    // Как результат - там будет мусор после вычисления
    FFTplanner::new(false)
        .plan_fft(buffer_fft.len())
        .process(input_fft_1, buffer_fft);

    // Надо помнить, что спектр симметричен относительно центра
    let process_fn =|(i, (val1, val2))| {
        let i = i as f32;            
        let freq = i * fft_sample_hz;

        let val1: &mut Complex32 = val1;
        let val2: &mut Complex32 = val2;

        // println!("{}hz = {}, {}", freq, val1, val2);

        // if val.norm() < 0.2 {
        //     val.set_zero()
        // }
        if freq > filter_freq {
            *val1 *= 0.0001;
        }
        *val2 = *val1;
    };

    // buffer_fft.iter().enumerate().for_each(|(i, val)| println!("{}: {:?}", i, val));

    let process_len = buffer_fft.len() / 2;
    let (left, right) = buffer_fft.split_at_mut(process_len+1);

    // Обработка от цетра к краю
    let mut temp1 = [Complex32::default()];
    let mut temp2 = [Complex32::default()];
    let right_iter = temp1
        .iter_mut()
        .chain(right
            .iter_mut()
            .rev())
        .chain(temp2
            .iter_mut());

    // Обработка от цетра к краю
    left
        .iter_mut()
        .zip(right_iter)
        .enumerate()
        .for_each(process_fn);

    // println!("");

    // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
    // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
    FFTplanner::new(true)
        .plan_fft(buffer_fft.len())
        .process(buffer_fft, input_fft_1);

    // Выполняем нормализацию
    let inv_len = 1.0 / (input_fft_1.len() as f32);
    input_fft_1
        .iter_mut()
        .for_each(|val|{
            *val *= inv_len;
        });
}

fn get_section_iterator_2<'a>(prev_input: &'a [f32], 
                              input: &'a [f32])-> impl std::iter::Iterator<Item=Complex32> + 'a {
    // let wind = 0.5 * (1.0 - ((2.0_f32 * std::f32::consts::PI * i as f32) / (window_size as f32 - 1.0_f32)).cos());
    let window = apodize::hanning_iter(prev_input.len());

    prev_input
        .iter()
        .skip(prev_input.len() / 2)
        .take(prev_input.len() / 2)
        .chain(input
            .iter()
            .take(input.len() / 2))
        .zip(window.map(|val| val as f32 ))
        .map(|(val, wind)|{
            Complex32::new(*val * wind, 0.0)
        })            
}

fn get_section_iterator_3<'a>(input: &'a [f32])-> impl std::iter::Iterator<Item=Complex32> + 'a {

    // let wind = 0.5 * (1.0 - ((2.0_f32 * std::f32::consts::PI * i as f32) / (window_size as f32 - 1.0_f32)).cos());
    let window = apodize::hanning_iter(input.len());

    input
        .iter()    
        .zip(window.map(|val| val as f32 ))
        .map(|(val, wind)|{
            Complex32::new(*val * wind, 0.0)
        })
}

fn crossfade_results<'a>(result_fft_1: &'a Vec<Complex32>, 
                         result_fft_2: &'a Vec<Complex32>, 
                         result_fft_3: &'a Vec<Complex32>, 
                         output: &'a mut [f32])-> impl Iterator<Item=(f32, &'a mut f32)>{

    let (out_2_left, out_2_right) = result_fft_2.split_at(output.len() / 2);

    // Итаратор с кроссфейдом
    let iter = result_fft_1
        .into_iter()
        .skip(output.len() / 2)
        .take(output.len() / 2)
        .zip(out_2_left
            .into_iter()
            .take(output.len() / 2))
        .map(|(val1, val2)|{
            val1.re + val2.re
        })
        .chain(out_2_right
            .into_iter()
            .zip(result_fft_3
                .into_iter()
                .take(output.len() / 2))
            .map(|(val1, val2)|{
                let val = val1.re + val2.re;
                val
            }))
        .zip(output.into_iter());

    iter
}

fn check_buffer_size<T>(buffer: &mut Vec<T>, size: usize)
where 
    T: Default,
    T: Clone
{
    if buffer.len() != size{
        buffer.resize(size, Default::default());
    }
}

fn update_with_iter(buffer: &mut Vec<Complex32>, iter: impl Iterator<Item=Complex32>){
    buffer
        .iter_mut()
        .zip(iter)
        .for_each(|(old, new)|{
            *old = new;
        });
}

plugin_main!(BasicPlugin); // Important!

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_process(){
        
    }
}