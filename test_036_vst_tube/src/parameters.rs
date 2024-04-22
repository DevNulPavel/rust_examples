use vst::util::AtomicFloat;
use vst::plugin::PluginParameters;


pub struct ComplexClipParams {
    pub threshold: AtomicFloat,
    pub lower_threshold: AtomicFloat,
    pub fold: AtomicFloat,
    pub gain: AtomicFloat,
}

impl Default for ComplexClipParams {
    fn default() -> ComplexClipParams {
        ComplexClipParams {
            threshold: AtomicFloat::new(1.0),
            lower_threshold: AtomicFloat::new(1.0),
            fold: AtomicFloat::new(0.0),
            gain: AtomicFloat::new(0.5),
        }
    }
}

impl PluginParameters for ComplexClipParams {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.threshold.get(),
            1 => self.lower_threshold.get(),
            2 => self.fold.get(),
            3 => self.gain.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.threshold.set(value.max(0.05)),
            1 => self.lower_threshold.set(value.max(0.05)),
            2 => self.fold.set(value.min(0.50)),
            3 => self.gain.set(value.max(0.01)),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Top half-wave threshold".to_string(),
            1 => "Bottom half-wave threshold".to_string(),
            2 => "fold".to_string(),
            3 => "gain".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", self.threshold.get() * 100.0),
            1 => format!("{}", self.lower_threshold.get() * 100.0),
            2 => format!("{}", self.fold.get() * 100.0),
            3 => format!("{}", self.gain.get() * 100.0),
            _ => "".to_string(),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 | 1 | 2 | 3 => "%".to_string(),
            _ => "".to_string(),
        }
    }
}