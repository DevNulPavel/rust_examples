use std::f32::consts::PI;
use std::sync::atomic::{ AtomicUsize, AtomicBool, Ordering };
use vst::util::AtomicFloat;
use vst::plugin::PluginParameters;

pub struct LadderParameters {
    // Частота среза
    pub cutoff: AtomicFloat,
    pub g: AtomicFloat,
    // Резонансный пик среза
    pub resonance: AtomicFloat,
    // Нужен для выбора степени среза и выбора вывода
    pub poles: AtomicUsize,
    // Используется для вывода параметра (pole_value is just to be able to use get_parameter on poles)
    pole_display_value: AtomicFloat,
    // Используется для повышения громкости, привода к перегрузу
    pub drive: AtomicFloat,
    // Режим среза верха или низа
    pub cut_mode: AtomicBool,
    cut_mode_value: AtomicFloat,
    // Частота семплирования нужна для вычисления среза
    pub sample_rate: AtomicFloat,
}

impl Default for LadderParameters {
    fn default() -> LadderParameters {
        LadderParameters {
            cutoff: AtomicFloat::new(1000.0),
            g: AtomicFloat::new(0.07135868),
            resonance: AtomicFloat::new(2.0),
            poles: AtomicUsize::new(3),
            pole_display_value: AtomicFloat::new(1.0),
            drive: AtomicFloat::new(0.0),
            cut_mode: AtomicBool::new(false),
            cut_mode_value: AtomicFloat::new(0.0),
            sample_rate: AtomicFloat::new(44100.0)
        }
    }
}

impl LadderParameters {
    pub fn set_cutoff(&self, value: f32) {
        // Частота среза выставляется от 0 до 20кГц
        // Формула среза устанавливает значение точно на низких частотах, и менее точно на высоких
        self.cutoff.set(20000. * (1.8f32.powf(10. * value - 10.)));
        // Билинейная трансформация дает нам очень точное значение среза
        self.g.set((PI * self.cutoff.get() / (self.sample_rate.get())).tan());
    }
    
    // Возвращает значение среза частоты для отображения (то есть значение от 0 до 1)
    pub fn get_cutoff(&self) -> f32 {
        1. + 0.17012975 * (0.00005 * self.cutoff.get()).ln()
    }

    // Установка значения порядка среза
    pub fn set_poles(&self, value: f32) {
        // Значение для интерфейса
        self.pole_display_value.set(value);
        // Фактическое значение
        self.poles.store(((value * 3.0).round()) as usize, Ordering::Relaxed);
    }

    pub fn set_mode(&self, value: f32){
        self.cut_mode_value.set(value);
        if value < 0.5 { 
            self.cut_mode.store(false, Ordering::Relaxed); 
        } else {
            self.cut_mode.store(true, Ordering::Relaxed)
        }
    }

    pub fn get_mode(&self)->f32{
        self.cut_mode_value.get()
    }
}

impl PluginParameters for LadderParameters {
    // Получаем значение параметра в диапазоне от 0 до 1
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.get_cutoff(),
            1 => self.resonance.get() / 4.0,
            2 => self.pole_display_value.get(),
            3 => self.drive.get() / 5.0,
            4 => self.get_mode(),
            _ => 0.0,
        }
    }

    // Установка значения в диапазоне от 0 до 1
    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.set_cutoff(value),
            1 => self.resonance.set(value * 4.0),
            2 => self.set_poles(value),
            3 => self.drive.set(value * 5.0),
            4 => self.set_mode(value),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "cutoff".to_string(),
            1 => "resonance".to_string(),
            2 => "filter order".to_string(),
            3 => "drive".to_string(),
            4 => "Low pass or high".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "Hz".to_string(),
            1 => "%".to_string(),
            2 => "poles".to_string(),
            3 => "%".to_string(),
            _ => "".to_string(),
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most sense.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.0}", self.cutoff.get()),
            1 => format!("{:.3}", self.resonance.get()),
            2 => format!("{}", self.poles.load(Ordering::Relaxed) + 1),
            3 => format!("{:.3}", self.drive.get()),
            4 => format!("{}", if self.cut_mode.load(Ordering::Relaxed) { "Low" } else { "High" } ),
            _ => format!(""),
        }
    }
}