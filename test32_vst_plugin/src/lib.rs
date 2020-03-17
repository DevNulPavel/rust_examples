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


////////////////////////////////////////////////////////////

struct SimplePluginParameters{
    volume: std::sync::Mutex<f32>,
    threshold: std::sync::Mutex<f32>
}

impl Default for SimplePluginParameters {
    fn default() -> SimplePluginParameters {
        SimplePluginParameters {
            volume: std::sync::Mutex::from(1.0_f32),
            threshold: std::sync::Mutex::from(1.0_f32)
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
                if let Ok(val) = self.threshold.lock() {
                    format!("{}", *val * 100.0)
                }else{
                    "".to_string()
                }
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
                if let Ok(val) = self.threshold.lock() {
                    *val
                }else{
                    0.0
                }
            },
            _ => 0.0,
        }
    }

    /// Установка значения параметра от 0 до 1, метод может быть вызван в потоке обработки данных для автоматизации
    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            // We don't want to divide by zero, so we'll clamp the value
            0 => {
                if let Ok(mut val) = self.threshold.lock() {
                    *val = value.max(0.01);
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

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let volume = if let Ok(val) = self.params.threshold.lock(){
            *val
        }else{
            0.0_f32
        };

        // split выдает нам пару значений входного и выходного буффера
        // Сейчас нам нужен толкьо выход, вход игнорируем
        let (_, mut output_buffer) = buffer.split();
    
        // Теперь итерируемся по значениям выходного буффера и заполняем их
        // Итерация сначала происходит по каналам, затем по данным
        for output_channel in output_buffer.into_iter() {
            // Итерируемся по данным канала
            for output_sample in output_channel {
                // Выдавать значения надо в диапазоне от -1.0 до 1.0
                *output_sample = (random::<f32>() - 0.5f32) * 2f32 * volume;
            }
        }
    } 

    // Выдаем ссылку на шареный объект параметров
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.params.clone()
    }
}

plugin_main!(BasicPlugin); // Important!