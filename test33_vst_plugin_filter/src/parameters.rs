use std::f32::consts::PI;
use std::sync::atomic::{ AtomicUsize, Ordering };
use vst::util::AtomicFloat;
use vst::plugin::PluginParameters;

pub struct LadderParameters {
    // the "cutoff" parameter. Determines how heavy filtering is
    pub cutoff: AtomicFloat,
    pub g: AtomicFloat,
    // needed to calculate cutoff.
    pub sample_rate: AtomicFloat,
    // makes a peak at cutoff
    pub res: AtomicFloat,
    // used to choose where we want our output to be
    pub poles: AtomicUsize,
    // pole_value is just to be able to use get_parameter on poles
    pub pole_value: AtomicFloat,
    // a drive parameter. Just used to increase the volume, which results in heavier distortion
    pub drive: AtomicFloat,
}

impl Default for LadderParameters {
    fn default() -> LadderParameters {
        LadderParameters {
            cutoff: AtomicFloat::new(1000.0),
            res: AtomicFloat::new(2.0),
            poles: AtomicUsize::new(3),
            pole_value: AtomicFloat::new(1.0),
            drive: AtomicFloat::new(0.0),
            sample_rate: AtomicFloat::new(44100.0),
            g: AtomicFloat::new(0.07135868),
        }
    }
}

impl LadderParameters {
    pub fn set_cutoff(&self, value: f32) {
        // cutoff formula gives us a natural feeling cutoff knob that spends more time in the low frequencies
        self.cutoff.set(20000. * (1.8f32.powf(10. * value - 10.)));
        // bilinear transformation for g gives us a very accurate cutoff
        self.g.set((PI * self.cutoff.get() / (self.sample_rate.get())).tan());
    }
    
    // returns the value used to set cutoff. for get_parameter function
    pub fn get_cutoff(&self) -> f32 {
        1. + 0.17012975 * (0.00005 * self.cutoff.get()).ln()
    }

    pub fn set_poles(&self, value: f32) {
        self.pole_value.set(value);
        self.poles.store(((value * 3.).round()) as usize, Ordering::Relaxed);
    }
}

impl PluginParameters for LadderParameters {
    // get_parameter has to return the value used in set_parameter
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.get_cutoff(),
            1 => self.res.get() / 4.,
            2 => self.pole_value.get(),
            3 => self.drive.get() / 5.,
            _ => 0.0,
        }
    }
    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.set_cutoff(value),
            1 => self.res.set(value * 4.),
            2 => self.set_poles(value),
            3 => self.drive.set(value * 5.),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "cutoff".to_string(),
            1 => "resonance".to_string(),
            2 => "filter order".to_string(),
            3 => "drive".to_string(),
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
            1 => format!("{:.3}", self.res.get()),
            2 => format!("{}", self.poles.load(Ordering::Relaxed) + 1),
            3 => format!("{:.3}", self.drive.get()),
            _ => format!(""),
        }
    }
}