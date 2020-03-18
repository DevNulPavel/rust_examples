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


////////////////////////////////////////////////////////////

struct SimplePluginParameters{
    volume: std::sync::Mutex<f32>,
    threshold: std::sync::Mutex<f32>
}

impl Default for SimplePluginParameters {
    fn default() -> SimplePluginParameters {
        SimplePluginParameters {
            volume: std::sync::Mutex::from(1.0_f32),
            threshold: std::sync::Mutex::from(1.0_f32),
        }
    }
}

impl PluginParameters for SimplePluginParameters {
    /// Изменение пресета, может быть вызван из потока обработки для автоматизации
    fn change_preset(&self, _preset: i32) {
    }

    /// Получаем номер текущего пресета
    fn get_preset_num(&self) -> i32 {
        0
    }

    /// Установка имени пресета
    fn set_preset_name(&self, _name: String) {}

    /// Получаем имя пресета по индексу
    fn get_preset_name(&self, _preset: i32) -> String {
        "".to_string()
    }

    /// Получаем имя параметра по индексу
    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "%".to_string(),
            1 => "%".to_string(),
            _ => "".to_string(),
        }
    }

    /// Получаем текстовое представление параметра по индексу
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            // Convert to a percentage
            0 => {
                let val = if let Ok(val) = self.threshold.lock(){
                    *val
                }else{
                    0.0_f32
                };
                format!("{}", val * 100.0)
            },
            1 => {
                let val = if let Ok(val) = self.volume.lock(){
                    *val
                }else{
                    0.0_f32
                };
                format!("{}", val * 100.0)
            },
            _ => "".to_string(),
        }
    }

    /// Получаем имя параметра по индексу
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Threshold".to_string(),
            1 => "Volume".to_string(),
            _ => "".to_string(),
        }
    }

    /// Получаем значения параметра по индексу, значение от 0 до 1
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => {
                let val = if let Ok(val) = self.threshold.lock(){
                    *val
                }else{
                    0.0_f32
                };
                val
            },
            1 => {
                let val = if let Ok(val) = self.volume.lock(){
                    *val
                }else{
                    0.0_f32
                };
                val
            },            
            _ => 0.0,
        }
    }

    /// Установка значения параметра от 0 до 1, метод может быть вызван в потоке обработки данных для автоматизации
    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            // We don't want to divide by zero, so we'll clamp the value
            0 => {
                if let Ok(mut val) = self.threshold.lock(){
                    *val = value;
                }
            },
            1 => {
                if let Ok(mut val) = self.volume.lock(){
                    *val = value;
                }
            },
            _ => (),
        }
    }

    /// Может ли быть параметр автоматизирован??
    fn can_be_automated(&self, _index: i32) -> bool {
        true
    }

    /// Use String as input for parameter value. Used by host to provide an editable field to
    /// adjust a parameter value. E.g. "100" may be interpreted as 100hz for parameter. Returns if
    /// the input string was used.
    fn string_to_parameter(&self, _index: i32, _text: String) -> bool {
        false
    }

    /// If `preset_chunks` is set to true in plugin info, this should return the raw chunk data for
    /// the current preset.
    fn get_preset_data(&self) -> Vec<u8> {
        Vec::new()
    }

    /// If `preset_chunks` is set to true in plugin info, this should return the raw chunk data for
    /// the current plugin bank.
    fn get_bank_data(&self) -> Vec<u8> {
        Vec::new()
    }

    /// If `preset_chunks` is set to true in plugin info, this should load a preset from the given
    /// chunk data.
    fn load_preset_data(&self, _data: &[u8]) {}

    /// If `preset_chunks` is set to true in plugin info, this should load a preset bank from the
    /// given chunk data.
    fn load_bank_data(&self, _data: &[u8]) {}
}

////////////////////////////////////////////////////////////

struct BasicPlugin{
    total_notes: i32,
    params: Arc<SimplePluginParameters>
}

impl Default for BasicPlugin {
    fn default() -> BasicPlugin {
        BasicPlugin {
            total_notes: 0,
            params: Arc::new(SimplePluginParameters::default())
        }
    }
}

impl Plugin for BasicPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "DevNul's Plugin".to_string(),
            unique_id: 13573312,        // Уникальный номер, чтобы отличать плагины
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
        let threshold = if let Ok(val) = self.params.threshold.lock(){
            *val
        }else{
            0.0_f32
        };
        let volume = if let Ok(val) = self.params.volume.lock(){
            *val
        }else{
            0.0_f32
        };

        // Создаем итератор по парам элементов, входа и выхода
        for (input, output) in buffer.zip() {
            // let const_val: f32 = input
            //     .iter()
            //     .fold(1.0_f32, |prev, new|{
            //         if new.abs() < prev.abs(){
            //             *new
            //         }else{
            //             prev
            //         }
            //     });
                // .map(|val| val.abs())
                // .min_by(|val1, val2|{
                //     if val1 < val2 {
                //         std::cmp::Ordering::Less
                //     }else if val1 > val2{
                //         std::cmp::Ordering::Greater
                //     }else{
                //         std::cmp::Ordering::Equal
                //     }
                // })
                // .unwrap_or(0.0_f32);

            let mut input_fft: Vec<Complex32> = input
                .iter()
                .flat_map(|val|{
                    std::iter::repeat(val)
                        .take(40)
                })
                .map(|val|{
                    Complex32::new(*val, 0.0)
                    // Complex32::new(*val - const_val, 0.0)
                    // if *val < 0.0_f32{
                    //     Complex32::new(*val - const_val.abs(), 0.0)
                    //     // Complex32::new(*val, 0.0)
                    // }else{
                    //     Complex32::new(*val - const_val.abs(), 0.0)
                    //     // Complex32::new(*val, 0.0)
                    // }
                })
                .collect();

            /*let from = output.len()*40 - output.len()/2;
            let to = output.len()*40 + output.len()/2;
            let mut input_fft: Vec<Complex32> = vec![Complex::zero(); output.len()*40];
            input_fft
                .iter_mut()
                .enumerate()
                .for_each(|(i, val)|{
                    if i > from && i <= to  {
                        *val = Complex32::new(input[i-from], 0.0);
                    }else{
                        *val = Complex32::new(0.0, 0.0)
                    }
                });*/

            let mut output_fft: Vec<Complex32> = vec![Complex::zero(); output.len()*40];

            // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
            // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
            let fft_to = FFTplanner::new(false).plan_fft(output_fft.len());

            // Обрабатываем данные
            // Входные данные мутабельные, так как они используются в качестве буффера
            // Как результат - там будет мусор после вычисления
            fft_to.process(&mut input_fft, &mut output_fft);
            
            let inv_len = 1.0 / (output_fft.len() as f32);
            let sqrt_len = 1.0 / inv_len.sqrt();
            // output_fft
            //     .iter_mut()
            //     .for_each(|val|{
            //         *val *= sqrt_len;
            //     });

            // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
            // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
            let fft_inv = FFTplanner::new(true).plan_fft(output_fft.len());

            fft_inv.process(&mut output_fft, &mut input_fft);

            input_fft
                .iter_mut()
                .for_each(|val|{
                    //val.norm();
                    *val *= inv_len;
                });

            // Для каждого входного и выходного семпла в буфферах
            for (in_sample, out_sample) in input_fft.into_iter().step_by(40).zip(output.into_iter()) {
                // let val = if in_sample.re < 0.0_f32{
                //     in_sample.re + min_abs
                // }else{
                //     in_sample.re + min_abs
                // };
                //let val = in_sample.re;
                let val: Complex32 = in_sample;
                let val = val.norm();
                // let val = val.re;
                // let val = val.norm() + const_val;

                *out_sample = val;
                //let val = *in_sample;

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
    } 
    
    // fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
    //     let volume = if let Ok(val) = self.params.volume.lock(){
    //         *val
    //     }else{
    //         0.0_f32
    //     };
    //     let threshold = if let Ok(val) = self.params.threshold.lock(){
    //         *val
    //     }else{
    //         0.0_f32
    //     };

    //     // let (_, mut output) = buffer.split();
    //     // for channel in 0..output.len() {
    //     //     let channel_data = output.get_mut(channel);
    //     //     for out_sample in channel_data{
    //     //         *out_sample = rand::random::<f32>() * volume;
    //     //     }
    //     // }
        
    //     // For each input and output
    //     for (input, output) in buffer.zip() {
    //         // For each input sample and output sample in buffer
    //         for (in_frame, out_frame) in input.iter().zip(output.iter_mut()) {
    //             //*out_frame = *in_frame * volume;
    //             //let random_power = (rand::random::<f32>() - 0.5) * 2.0 * threshold;
    //             let random_power = rand::random::<f32>() * threshold;
    //             *out_frame = (in_frame * volume * random_power).min(1.0).max(-1.0);
    //         }
    //     }
    // }

    // Выдаем ссылку на шареный объект параметров
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.params.clone()
    }
}

plugin_main!(BasicPlugin); // Important!