//! This zero-delay feedback filter is based on a 4-stage transistor ladder filter.
//! It follows the following equations:
//! x = input - tanh(self.res * self.vout[3])
//! vout[0] = self.params.g.get() * (tanh(x) - tanh(self.vout[0])) + self.s[0]
//! vout[1] = self.params.g.get() * (tanh(self.vout[0]) - tanh(self.vout[1])) + self.s[1]
//! vout[0] = self.params.g.get() * (tanh(self.vout[1]) - tanh(self.vout[2])) + self.s[2]
//! vout[0] = self.params.g.get() * (tanh(self.vout[2]) - tanh(self.vout[3])) + self.s[3]
//! since we can't easily solve a nonlinear equation,
//! Mystran's fixed-pivot method is used to approximate the tanh() parts.
//! Quality can be improved a lot by oversampling a bit.
//! Feedback is clipped independently of the input, so it doesn't disappear at high gains.

mod parameters;

use std::sync::Arc;

use vst::plugin_main;
use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use crate::parameters::LadderParameters;

// this is a 4-pole filter with resonance, which is why there's 4 states and vouts
#[derive(Clone)]
struct LadderFilter {
    // Параметры данного фильтра
    params: Arc<LadderParameters>,
    // Значения для фильтра
    buf_0: f32,
    buf_1: f32,
}

// Методы фильтра
impl LadderFilter {
}

impl Default for LadderFilter {
    fn default() -> LadderFilter {
        LadderFilter {
            buf_0: 0.0f32,
            buf_1: 0.0f32,
            params: Arc::new(LadderParameters::default()),
        }
    }
}

impl Plugin for LadderFilter {
    // Сохраняем частоту семплирования
    fn set_sample_rate(&mut self, rate: f32) {
        self.params.sample_rate.set(rate);
    }

    fn get_info(&self) -> Info {
        Info {
            name: "DevNul's filter 2".to_string(),
            unique_id: 4564564,
            inputs: 1,
            outputs: 1,
            category: Category::Effect,
            parameters: 3,
            ..Default::default()
        }
    }

    // https://habr.com/ru/post/227791/
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let cutoff = self.params.get_cutoff();
        let gain = self.params.get_gain() * 10.0;

        // Обходим буфферы каждого из каналов
        for (input_buffer, output_buffer) in buffer.zip() {
            // Обходим семплы из каналов
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {

                self.buf_0 += cutoff * (input_sample - self.buf_0);
                self.buf_1 += cutoff * (self.buf_0 - self.buf_1);
                
                /*switch (mode) {
                    case FILTER_MODE_LOWPASS:
                        return buf1;
                    case FILTER_MODE_HIGHPASS:
                        return inputValue - buf0;
                    case FILTER_MODE_BANDPASS:
                        return buf0 - buf1;
                    default:
                        return 0.0;
                }*/
                
                *output_sample = (input_sample - self.buf_0) * gain;
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

plugin_main!(LadderFilter);
