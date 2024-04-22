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

use crate::parameters::ComplexClipParams;

struct ComplexClip {
    params: Arc<ComplexClipParams>,
}

impl Default for ComplexClip {
    fn default() -> ComplexClip {
        ComplexClip {
            params: Arc::new(ComplexClipParams::default()),
        }
    }
}

impl Plugin for ComplexClip {
    fn get_info(&self) -> Info {
        Info {
            name: "DevNul's distortion".to_string(),
            vendor: "DevNul".to_string(),
            unique_id: 245435,

            inputs: 2,
            outputs: 2,
            parameters: 4,
            category: Category::Effect,

            ..Info::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let threshold = self.params.threshold.get();
        let lower_threshold = self.params.lower_threshold.get();
        let fold = self.params.fold.get();
        let gain = self.params.gain.get();

        buffer.zip().for_each(|(input_buffer, output_buffer)| {
            input_buffer
                .iter()
                .zip(output_buffer)
                .for_each(|(input_sample, output_sample)| {
                    // Положительная ли полуволна?
                    let positive = *input_sample >= 0.0;

                    // Начальное значение, если положительная полуволна
                    // Тогда ограничиваем верхнее значение полуволны верхним порогом
                    // Если нет - тогда нижним порогом
                    let starting_value = if positive == true {
                        input_sample.min(threshold)
                    } else {
                        input_sample.max(-lower_threshold)
                    };
                    
                    // Нужно ли клипповать или нет
                    let clipped = if positive == true {
                        input_sample > &threshold
                    } else {
                        input_sample < &lower_threshold
                    };

                    *output_sample = if clipped == true {
                        // Если надо клипповать значение и верхняя полуволна
                        if positive == true {
                            // Разница между входным значением и порогом
                            let difference = input_sample - threshold;
                            // Получаем конечное значение
                            ((starting_value - (difference * fold)) / threshold) * gain
                        } else {
                            // Разница между входным значением и порогом
                            let difference = input_sample + lower_threshold;
                            // Получаем конечное значение
                            ((starting_value - (difference * fold)) / lower_threshold) * gain
                        }
                    } else {
                        // Если нет клиппинга - выдаем значение как есть
                        if positive == true {
                            (starting_value / threshold) * gain
                        } else {
                            (starting_value / lower_threshold) * gain
                        }
                    };
                });
        });
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

plugin_main!(ComplexClip);