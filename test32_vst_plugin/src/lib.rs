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
    threshold: std::sync::atomic::AtomicU32
}

impl Default for SimplePluginParameters {
    fn default() -> SimplePluginParameters {
        SimplePluginParameters {
            volume: std::sync::Mutex::from(1.0_f32),
            threshold: std::sync::atomic::AtomicU32::from(1_000_000)
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
            _ => "".to_string(),
        }
    }

    /// Получаем текстовое представление параметра по индексу
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            // Convert to a percentage
            0 => {
                // let val = self.threshold.load(std::sync::atomic::Ordering::Acquire);
                // let val = val as f32 / 1_000_000.0_f32;
                let val = 1.0_f32;
                format!("{}", val * 100.0)
            }
            _ => "".to_string(),
        }
    }

    /// Получаем имя параметра по индексу
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Threshold".to_string(),
            _ => "".to_string(),
        }
    }

    /// Получаем значения параметра по индексу, значение от 0 до 1
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => {
                //let val = self.threshold.load(std::sync::atomic::Ordering::Acquire);
                //let val = val as f32 / 1_000_000.0_f32;
                //val
                1.0_f32
            },
            _ => 0.0,
        }
    }

    /// Установка значения параметра от 0 до 1, метод может быть вызван в потоке обработки данных для автоматизации
    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            // We don't want to divide by zero, so we'll clamp the value
            0 => {
                //let val = (value * 1_000_000.0_f32) as u32;
                //self.threshold.store(val, std::sync::atomic::Ordering::Acquire);
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
            unique_id: 1357,            // Уникальный номер, чтобы отличать плагины
            presets: 0,                 // Количество пресетов
            inputs: 1,                  // Сколько каналов звука на входе
            outputs: 1,                 // Каналы звука на выходе
            version: 0001,              // Версия плагина 
            category: Category::Synth,  // Тип плагина
            parameters: 1,
            //initial_delay, 
            //preset_chunks, 
            //f64_precision, 
            //silent_when_stopped

            ..Default::default()
        }
    }

    // Функция, которая вызывается на события, такие как MIDI и тд
    fn process_events(&mut self, events: &Events) {
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
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>){
        //let val = self.params.threshold.load(std::sync::atomic::Ordering::Acquire);
        //let threshold = val as f32 / 1_000_000.0_f32;
        let threshold = 0.9_f32;

        // Создаем итератор по парам элементов, входа и выхода
        for (input, output) in buffer.zip() {
            let const_val: f32 = input
                .iter()
                .fold(1.0_f32, |prev, new|{
                    if new.abs() < prev.abs(){
                        *new
                    }else{
                        prev
                    }
                });
                /*.map(|val| val.abs())
                .min_by(|val1, val2|{
                    if val1 < val2 {
                        std::cmp::Ordering::Less
                    }else if val1 > val2{
                        std::cmp::Ordering::Greater
                    }else{
                        std::cmp::Ordering::Equal
                    }
                })
                .unwrap_or(0.0_f32);*/

            // FFTplanner позволяет выбирать оптимальный алгоритм работы для входного размера данных
            let mut planner = FFTplanner::new(false);

            // Создаем объект, который содержит в себе оптимальный алгоритм преобразования фурье
            let fft = planner.plan_fft(input.len());

            let mut input_fft: Vec<Complex32> = input
                .iter()
                .map(|val|{
                    Complex32::new(*val, 0.0)
                    //Complex32::new(*val - const_val, 0.0)
                    /*if *val < 0.0_f32{
                        Complex32::new(*val - const_val.abs(), 0.0)
                        // Complex32::new(*val, 0.0)
                    }else{
                        Complex32::new(*val - const_val.abs(), 0.0)
                        // Complex32::new(*val, 0.0)
                    }*/
                })
                .collect();

            let mut output_fft: Vec<Complex32> = vec![Complex::zero(); output.len()];

            // Обрабатываем данные
            // Входные данные мутабельные, так как они используются в качестве буффера
            // Как результат - там будет мусор после вычисления
            fft.process(&mut input_fft, &mut output_fft);
            
            let sqrt_len = 1.0 / (output_fft.len() as f32).sqrt();
            output_fft
                .iter_mut()
                .for_each(|val|{
                    *val *= sqrt_len;
                });

            fft.process(&mut output_fft, &mut input_fft);

            input_fft
                .iter_mut()
                .for_each(|val|{
                    *val *= sqrt_len;
                });


            // Для каждого входного и выходного семпла в буфферах
            for (in_sample, out_sample) in input_fft.into_iter().zip(output.into_iter()) {
                /*let val = if in_sample.re < 0.0_f32{
                    in_sample.re + min_abs
                }else{
                    in_sample.re + min_abs
                };*/
                let val = in_sample.re;
                // let val = in_sample.re + const_val;

                // Эмулируем клиппинг значений
                *out_sample = if val > threshold {
                    threshold
                } else if val < -threshold {
                    -threshold
                } else {
                    val
                };
            }
        }
    }

    // Выдаем ссылку на шареный объект параметров
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.params.clone()
    }
}

plugin_main!(BasicPlugin); // Important!