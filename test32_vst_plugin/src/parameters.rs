use std::sync::Mutex;
use vst::plugin::{
    PluginParameters
};

pub(super) struct SimplePluginParameters{
    volume: Mutex<f32>,
    threshold: Mutex<f32>
}

impl Default for SimplePluginParameters {
    fn default() -> SimplePluginParameters {
        SimplePluginParameters {
            volume: Mutex::from(1.0_f32),
            threshold: Mutex::from(1.0_f32),
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

impl SimplePluginParameters{
    pub(super) fn get_threshold(&self) -> f32 {
        if let Ok(val) = self.threshold.lock(){
            *val
        }else{
            0.0_f32
        }
    }

    pub(super) fn get_volume(&self) -> f32 {
        if let Ok(val) = self.volume.lock(){
            *val
        }else{
            0.0_f32
        }
    }
}