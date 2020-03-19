//! Фильтр обратной связи с нулевой задержкой, основанный на четырех-стадийном фильтре ladder
//! Он выполняет следующие вычисления:
//! x = input - tanh(resonance * out[3])
//! out[0] = self.params.g.get() * (tanh(x) - tanh(self.vout[0])) + self.s[0]
//! out[1] = self.params.g.get() * (tanh(self.vout[0]) - tanh(self.vout[1])) + self.s[1]
//! out[0] = self.params.g.get() * (tanh(self.vout[1]) - tanh(self.vout[2])) + self.s[2]
//! out[0] = self.params.g.get() * (tanh(self.vout[2]) - tanh(self.vout[3])) + self.s[3]
//! Мы решаем просто нелинейное уранение
//! Метод мистрана с фиксированной опорной точно исползуется для приблизительного вычисления tanh частей
//! Качествоможет быть улучшено с помощью оверсмплинга?
//! Обратная связь клиппует внезависимости от входа, поэтому не скрыть за высоким гейном

mod parameters;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use vst::plugin_main;
use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use crate::parameters::LadderParameters;

// Четрых-стадийным фильтр с резонансом, поэтому тут 4 состояния и выходных значения
#[derive(Clone)]
struct LadderFilter {
    // Параметры данного фильтра
    params: Arc<LadderParameters>,
    // Индекс текущего обрабатываемого канала
    active_channel: usize,
    // Выходные значения каждой из стадий фильтра
    vout: [[f32; 4]; 2],
    // Параметр состояния. В IIR это может быть последнее значени из фильтра
    // Используется для трапезиоидного интегрирования длябы избежать задержки
    last_state: [[f32; 4]; 2],
}

// Методы фильтра
impl LadderFilter {
    fn set_active_channel(&mut self, ch: usize){
        match ch{
            ch if (ch < 2) => self.active_channel = ch,
            _ => self.active_channel = 0,
        }
    }

    // После каждой обработки нужно обновлять состояние, используется для интегрирования для исключения задержки
    fn update_state(&mut self) {
        let s = &mut self.last_state[self.active_channel];
        let out = &self.vout[self.active_channel];

        // Сохраняем состояние в виде (2 * получившийся выход) - прошлое состояние
        s[0] = 2.0 * out[0] - s[0];
        s[1] = 2.0 * out[1] - s[1];
        s[2] = 2.0 * out[2] - s[2];
        s[3] = 2.0 * out[3] - s[3];
    }

    // Выполняет процесс фильтрации (mystran's method)
    fn tick_pivotal(&mut self, input: f32) {
        if self.params.drive.get() > 0. {
            self.run_ladder_nonlinear(input * (self.params.drive.get() + 0.7));
        } else {
            //
            self.run_ladder_linear(input);
        }

        // Обновляем состояние для следующей итерации
        self.update_state();
    }

    // Нелинейная функция фитльтрации ladder с искажением
    fn run_ladder_nonlinear(&mut self, input: f32) {
        let s = &self.last_state[self.active_channel];
        let out = &mut self.vout[self.active_channel];
        let g = self.params.g.get();
        let resonance = self.params.resonance.get();

        let mut a = [1f32; 5];
        let base = [input, s[0], s[1], s[2], s[3]];
        // a[n] is the fixed-pivot approximation for tanh()
        for n in 0..base.len() {
            if base[n] != 0. {
                a[n] = base[n].tanh() / base[n];
            } else {
                a[n] = 1.;
            }
        }
        // denominators of solutions of individual stages. Simplifies the math a bit
        let g0 = 1. / (1. + g * a[1]);
        let g1 = 1. / (1. + g * a[2]);
        let g2 = 1. / (1. + g * a[3]);
        let g3 = 1. / (1. + g * a[4]);
        //  these are just factored out of the feedback solution. Makes the math way easier to read
        let f3 = g * a[3] * g3;
        let f2 = g * a[2] * g2 * f3;
        let f1 = g * a[1] * g1 * f2;
        let f0 = g * g0 * f1;
        // outputs a 24db filter
        out[3] =
            (f0 * input * a[0] + f1 * g0 * s[0] + f2 * g1 * s[1] + f3 * g2 * s[2] + g3 * s[3])
                / (f0 * resonance * a[3] + 1.);
        // since we know the feedback, we can solve the remaining outputs:
        out[0] = g0 * (g * a[1] * (input * a[0] - resonance * a[3] * out[3]) + s[0]);
        out[1] = g1 * (g * a[2] * out[0] + s[1]);
        out[2] = g2 * (g * a[3] * out[1] + s[2]);
    }

    // Линейный вариант без искажений
    pub fn run_ladder_linear(&mut self, input: f32) {
        let s = &self.last_state[self.active_channel];
        let out = &mut self.vout[self.active_channel];
        let g = self.params.g.get();
        let resonance = self.params.resonance.get();

        // Знаменатель для вычисления отдельных этапов. Упрощение математики ниже
        let g0 = 1.0 / (1.0 + g);     // 1 / (1 + частота среза)
        let g1 = g * g0 * g0;
        let g2 = g * g1 * g0;
        let g3 = g * g2 * g0;

        // outputs a 24db filter
        out[3] = (g3 * g * input + g0 * s[3] + g1 * s[2] + g2 * s[1] + g3 * s[0])
                    / (g3 * g * resonance + 1.0);
        // Так как мы знаем feedback, мы можем вычислить оставшийся вывод
        out[0] = g0 * (g * (input - resonance * out[3]) + s[0]);
        out[1] = g0 * (g * out[0] + s[1]);
        out[2] = g0 * (g * out[1] + s[2]);
    }
}

impl Default for LadderFilter {
    fn default() -> LadderFilter {
        LadderFilter {
            vout: [[0_f32; 4]; 2],
            last_state: [[0_f32; 4]; 2],
            active_channel: 0,
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
            name: "DevNul's poly filter".to_string(),
            vendor: "DevNul".to_string(),
            unique_id: 645345,
            inputs: 2,
            outputs: 2,
            category: Category::Effect,
            parameters: 5,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let mode = self.params.cut_mode.load(Ordering::Relaxed);
        let poles = self.params.poles.load(Ordering::Relaxed);

        let mut channel: usize = 0;
        // Обходим буфферы каждого из каналов
        for (input_buffer, output_buffer) in buffer.zip() {
            self.set_active_channel(channel);

            // Обходим семплы из каналов
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                self.tick_pivotal(*input_sample);

                let out = &self.vout[channel];

                // Получаем нужные значения в зависимости от количества стадий фильтрации
                if mode{
                    // Фильтр верхних частот
                    *output_sample = out[poles];
                }else{
                    // Фильтр нижних частот
                    *output_sample = input_sample - out[poles];
                }
            }

            channel += 1;
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

plugin_main!(LadderFilter);
