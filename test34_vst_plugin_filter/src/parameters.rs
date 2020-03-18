use vst::util::AtomicFloat;
use vst::plugin::PluginParameters;



pub struct LadderParameters {
    // Частота семплирования
    pub sample_rate: AtomicFloat,
    // Частота среза
    pub cutoff: AtomicFloat,
    // Gain
    pub gain: AtomicFloat,
    // Резонанс
    pub resonance: AtomicFloat,
}

impl Default for LadderParameters {
    fn default() -> LadderParameters {
        LadderParameters {
            sample_rate: AtomicFloat::new(44100.0),
            cutoff: AtomicFloat::new(1000.0),
            gain: AtomicFloat::new(0.1),
            resonance: AtomicFloat::new(2.0),           
        }
    }
}

impl LadderParameters {
    /// Установка частоты среза, параметр от 0 до 1
    pub fn set_cutoff(&self, value: f32) {
        // cutoff formula gives us a natural feeling cutoff knob that spends more time in the low frequencies
        // Формула дает нам нормальную становку частоты среза, когда внизу больше точность, чем внизу
        self.cutoff.set(20000. * (1.8f32.powf(10. * value - 10.)));
    }
    
    // returns the value used to set cutoff. for get_parameter function
    // параметр от 0 до 1
    pub fn get_cutoff(&self) -> f32 {
        // Получаем частоту среза обратно
        1. + 0.17012975 * (0.00005 * self.cutoff.get()).ln()
    }

    /// Установка частоты среза, параметр от 0 до 1
    pub fn set_gain(&self, value: f32) {
        self.gain.set(value);
    }
    
    // returns the value used to set cutoff. for get_parameter function
    // параметр от 0 до 1
    pub fn get_gain(&self) -> f32 {
        self.gain.get()
    }
}

impl PluginParameters for LadderParameters {
    // get_parameter has to return the value used in set_parameter
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.get_cutoff(),
            1 => self.resonance.get() / 4.,
            2 => self.get_gain(),
            _ => 0.0,
        }
    }
    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.set_cutoff(value),
            1 => self.resonance.set(value * 4.),
            2 => self.set_gain(value),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "cutoff".to_string(),
            1 => "resonance".to_string(),
            2 => "gain".to_string(),
            _ => "".to_string(),
        }
    }
    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "Hz".to_string(),
            1 => "%".to_string(),
            2 => "%".to_string(),
            _ => "".to_string(),
        }
    }
    // This is what will display underneath our control.  We can
    // format it into a string that makes the most sense.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.0}", self.cutoff.get()),
            1 => format!("{:.3}", self.resonance.get()),
            2 => format!("{:.1}", self.gain.get() * 10.0 * 100.0),
            _ => format!(""),
        }
    }
}