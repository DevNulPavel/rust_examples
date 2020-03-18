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

use std::sync::atomic::Ordering;
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
    // Выходные значения каждой из стадий фильтра
    vout: [f32; 4],
    // Параметр состояния. В IIR это может быть последнее значени из фильтра
    // Используется для трапезиоидного интегрирования длябы избежать задержки
    s: [f32; 4],
}

// Методы фильтра
impl LadderFilter {
    // После каждой обработки нужно обновлять состояние, используется для интегрирования для исключения задержки
    fn update_state(&mut self) {
        self.s[0] = 2. * self.vout[0] - self.s[0];
        self.s[1] = 2. * self.vout[1] - self.s[1];
        self.s[2] = 2. * self.vout[2] - self.s[2];
        self.s[3] = 2. * self.vout[3] - self.s[3];
    }

    // Выполняет процесс фильтрации (mystran's method)
    fn tick_pivotal(&mut self, input: f32) {
        if self.params.drive.get() > 0. {
            self.run_ladder_nonlinear(input * (self.params.drive.get() + 0.7));
        } else {
            //
            self.run_ladder_linear(input);
        }
        self.update_state();
    }

    // Нелинейная функция фитльтрации ladder с искажением
    fn run_ladder_nonlinear(&mut self, input: f32) {
        let mut a = [1f32; 5];
        let base = [input, self.s[0], self.s[1], self.s[2], self.s[3]];
        // a[n] is the fixed-pivot approximation for tanh()
        for n in 0..base.len() {
            if base[n] != 0. {
                a[n] = base[n].tanh() / base[n];
            } else {
                a[n] = 1.;
            }
        }
        // denominators of solutions of individual stages. Simplifies the math a bit
        let g0 = 1. / (1. + self.params.g.get() * a[1]);
        let g1 = 1. / (1. + self.params.g.get() * a[2]);
        let g2 = 1. / (1. + self.params.g.get() * a[3]);
        let g3 = 1. / (1. + self.params.g.get() * a[4]);
        //  these are just factored out of the feedback solution. Makes the math way easier to read
        let f3 = self.params.g.get() * a[3] * g3;
        let f2 = self.params.g.get() * a[2] * g2 * f3;
        let f1 = self.params.g.get() * a[1] * g1 * f2;
        let f0 = self.params.g.get() * g0 * f1;
        // outputs a 24db filter
        self.vout[3] =
            (f0 * input * a[0] + f1 * g0 * self.s[0] + f2 * g1 * self.s[1] + f3 * g2 * self.s[2] + g3 * self.s[3])
                / (f0 * self.params.res.get() * a[3] + 1.);
        // since we know the feedback, we can solve the remaining outputs:
        self.vout[0] = g0
            * (self.params.g.get() * a[1] * (input * a[0] - self.params.res.get() * a[3] * self.vout[3]) + self.s[0]);
        self.vout[1] = g1 * (self.params.g.get() * a[2] * self.vout[0] + self.s[1]);
        self.vout[2] = g2 * (self.params.g.get() * a[3] * self.vout[1] + self.s[2]);
    }

    // Линейный вариант без искажений
    pub fn run_ladder_linear(&mut self, input: f32) {
        // denominators of solutions of individual stages. Simplifies the math a bit
        let g0 = 1.0 / (1.0 + self.params.g.get());
        let g1 = self.params.g.get() * g0 * g0;
        let g2 = self.params.g.get() * g1 * g0;
        let g3 = self.params.g.get() * g2 * g0;
        // outputs a 24db filter
        self.vout[3] =
            (g3 * self.params.g.get() * input + g0 * self.s[3] + g1 * self.s[2] + g2 * self.s[1] + g3 * self.s[0])
                / (g3 * self.params.g.get() * self.params.res.get() + 1.0);
        // since we know the feedback, we can solve the remaining outputs:
        self.vout[0] = g0 * (self.params.g.get() * (input - self.params.res.get() * self.vout[3]) + self.s[0]);
        self.vout[1] = g0 * (self.params.g.get() * self.vout[0] + self.s[1]);
        self.vout[2] = g0 * (self.params.g.get() * self.vout[1] + self.s[2]);
    }
}

impl Default for LadderFilter {
    fn default() -> LadderFilter {
        LadderFilter {
            vout: [0f32; 4],
            s: [0f32; 4],
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
            name: "DevNul's filter".to_string(),
            unique_id: 926378,
            inputs: 1,
            outputs: 1,
            category: Category::Effect,
            parameters: 4,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // Обходим буфферы каждого из каналов
        for (input_buffer, output_buffer) in buffer.zip() {
            // Обходим семплы из каналов
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                self.tick_pivotal(*input_sample);

                // Получаем нужные значения в зависимости от количества стадий фильтрации
                *output_sample = self.vout[self.params.poles.load(Ordering::Relaxed)];
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

plugin_main!(LadderFilter);
