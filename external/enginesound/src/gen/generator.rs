use crate::recorder::Recorder;
use crate::gen::intake_valve;
use crate::gen::engine::Engine;
use crate::gen::low_pass_filter::LowPassFilter;

pub struct Generator {
    pub recorder: Option<Recorder>,
    pub volume: f32,
    pub samples_per_second: u32,
    pub engine: Engine,
    /// `LowPassFilter` which is subtracted from the sample while playing back to reduce dc offset and thus clipping
    dc_lp: LowPassFilter,
    /// set to true by any waveguide if it is dampening it's output to prevent feedback loops
    pub waveguides_dampened: bool,
    /// set to true if the amplitude of the recording is greater than 1
    pub recording_currently_clipping: bool,
}

impl Generator {
    pub(crate) fn new(samples_per_second: u32, engine: Engine, dc_lp: LowPassFilter) -> Generator {
        Generator {
            recorder: None,
            volume: 0.4_f32,
            samples_per_second,
            engine,
            dc_lp,
            waveguides_dampened: false,
            recording_currently_clipping: false,
        }
    }

    pub(crate) fn generate(&mut self, buf: &mut [f32]) {
        // Текущая позиция коленвала
        let crankshaft_pos = self.engine.crankshaft_pos;

        // TODO: почему 120?
        // Количество семплов * 120
        let samples_per_second = self.samples_per_second as f32 * 120.0;

        // Обнуляем флаги клиппинга и ???
        self.recording_currently_clipping = false;
        self.waveguides_dampened = false;

        // TODO: Итератор?
        // Заполнение выходного буфера значениями
        let mut i = 1.0;
        let mut ii = 0;
        while ii < buf.len() {
            // Обновляем позицию коленвала
            // Fract возвращает дробную часть позиции коленвала
            // То есть значение будет от 0 до 1
            self.engine.crankshaft_pos = (crankshaft_pos + i * self.get_rpm() / samples_per_second).fract();

            // Генерируем новый семпл
            let (intake, engine_vibrations, exhaust, waveguides_dampened) = self.gen();

            // Получаем значение суммированием
            let sum_values = 
                intake * self.get_intake_volume() +
                engine_vibrations * self.get_engine_vibrations_volume() +
                exhaust * self.get_exhaust_volume();
            let sample = sum_values * self.get_volume();
            
            // Заглушены ли значения
            self.waveguides_dampened |= waveguides_dampened;

            // Фильтруем низкочастотные шумы (DC offsed) и записываем значение в выходной буфер
            buf[ii] = sample - self.dc_lp.filter(sample);

            // i++
            i += 1.0;
            ii += 1;
        }

        // Если надо записывать
        if let Some(recorder) = &mut self.recorder {
            // Тогда буфер в вектор
            let bufvec = buf.to_vec();
            
            // Флаг наличия клипинга, если хотя бы у одного есть - значит true выставляем
            self.recording_currently_clipping = bufvec
                .iter()
                .any(|sample| {
                    sample.abs() > 1.0
                });

            // Записываем наш вектор
            recorder.record(bufvec);
        }
    }

    pub fn reset(&mut self) {
        for cyl in self.engine.cylinders.iter_mut() {
            cyl.exhaust_waveguide
                .chamber0
                .samples
                .data
                .iter_mut()
                .for_each(|sample| *sample = 0.0);
            cyl.exhaust_waveguide
                .chamber1
                .samples
                .data
                .iter_mut()
                .for_each(|sample| *sample = 0.0);
            cyl.intake_waveguide
                .chamber0
                .samples
                .data
                .iter_mut()
                .for_each(|sample| *sample = 0.0);
            cyl.intake_waveguide
                .chamber1
                .samples
                .data
                .iter_mut()
                .for_each(|sample| *sample = 0.0);
            cyl.extractor_waveguide
                .chamber0
                .samples
                .data
                .iter_mut()
                .for_each(|sample| *sample = 0.0);
            cyl.extractor_waveguide
                .chamber1
                .samples
                .data
                .iter_mut()
                .for_each(|sample| *sample = 0.0);

            cyl.extractor_exhaust = 0.0;
            cyl.cyl_sound = 0.0;
        }

        self.engine
            .muffler
            .straight_pipe
            .chamber0
            .samples
            .data
            .iter_mut()
            .for_each(|sample| *sample = 0.0);
        self.engine
            .muffler
            .straight_pipe
            .chamber1
            .samples
            .data
            .iter_mut()
            .for_each(|sample| *sample = 0.0);

        self.engine
            .engine_vibration_filter
            .samples
            .data
            .iter_mut()
            .for_each(|sample| *sample = 0.0);
        self.engine
            .engine_vibration_filter
            .samples
            .data
            .iter_mut()
            .for_each(|sample| *sample = 0.0);

        self.engine
            .crankshaft_fluctuation_lp
            .samples
            .data
            .iter_mut()
            .for_each(|sample| *sample = 0.0);
        self.engine
            .crankshaft_fluctuation_lp
            .samples
            .data
            .iter_mut()
            .for_each(|sample| *sample = 0.0);

        for muffler_element in self.engine.muffler.muffler_elements.iter_mut() {
            muffler_element
                .chamber0
                .samples
                .data
                .iter_mut()
                .for_each(|sample| *sample = 0.0);
            muffler_element
                .chamber1
                .samples
                .data
                .iter_mut()
                .for_each(|sample| *sample = 0.0);
        }

        self.engine.exhaust_collector = 0.0;
        self.engine.intake_collector = 0.0;
    }

    #[inline]
    pub fn get_rpm(&self) -> f32 {
        self.engine.rpm
    }

    #[inline]
    pub fn set_rpm(&mut self, rpm: f32) {
        self.engine.rpm = rpm;
    }

    #[inline]
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    #[inline]
    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    #[inline]
    pub fn set_intake_volume(&mut self, intake_volume: f32) {
        self.engine.intake_volume = intake_volume;
    }

    #[inline]
    pub fn get_intake_volume(&self) -> f32 {
        self.engine.intake_volume
    }

    #[inline]
    pub fn set_exhaust_volume(&mut self, exhaust_volume: f32) {
        self.engine.exhaust_volume = exhaust_volume;
    }

    #[inline]
    pub fn get_exhaust_volume(&self) -> f32 {
        self.engine.exhaust_volume
    }

    #[inline]
    pub fn set_engine_vibrations_volume(&mut self, engine_vibrations_volume: f32) {
        self.engine.engine_vibrations_volume = engine_vibrations_volume;
    }

    #[inline]
    pub fn get_engine_vibrations_volume(&self) -> f32 {
        self.engine.engine_vibrations_volume
    }

    /// Генерирует значения для текущего состояния коленвала
    /// возвращает `(intake, engine vibrations, exhaust, waveguides dampened)`
    fn gen(&mut self) -> (f32, f32, f32, bool) {
        // Шум от впуска двигателя
        let intake_noise = self.engine.intake_noise.step();
        let intake_noise = self
            .engine
            .intake_noise_lp
            .filter(intake_noise) * self.engine.intake_noise_factor;

        // Вибрации
        let mut engine_vibration = 0.0;

        // Сколько цилиндров
        let num_cyl = self.engine.cylinders.len() as f32;

        // Выпускной коллектор
        let last_exhaust_collector = self.engine.exhaust_collector / num_cyl;

        // Обнуляем значения сумм звука выпуска и впуска
        self.engine.exhaust_collector = 0.0;
        self.engine.intake_collector = 0.0;

        // Смещение колебаний коленвала
        let crankshaft_fluctuation_offset = self.engine.crankshaft_noise.step();
        let crankshaft_fluctuation_offset = self
            .engine
            .crankshaft_fluctuation_lp
            .filter(crankshaft_fluctuation_offset);

        // Цилиндры затушены?
        let mut cylinder_dampened = false;

        // Идем по всем цилиндрам
        for cylinder in self.engine.cylinders.iter_mut() {
            let crankshaft_pos = self.engine.crankshaft_pos + self.engine.crankshaft_fluctuation * crankshaft_fluctuation_offset;

            // Получаем значения звуков от цилиндра
            let (cyl_intake, cyl_exhaust, cyl_vib, dampened) = cylinder.pop(
                crankshaft_pos,
                last_exhaust_collector,
                self.engine.intake_valve_shift,
                self.engine.exhaust_valve_shift,
            );

            // Суммируем значение
            self.engine.intake_collector += cyl_intake;
            self.engine.exhaust_collector += cyl_exhaust;

            // Добавляем вибрации
            engine_vibration += cyl_vib;

            // Был ли заглушен?
            cylinder_dampened |= dampened;
        }

        // parallel input to the exhaust straight pipe
        // alpha end is at exhaust collector
        let straight_pipe_wg_ret = self.engine.muffler.straight_pipe.pop();

        // alpha end is at straight pipe end (beta)
        let mut muffler_wg_ret = (0.0, 0.0, false);

        for muffler_line in self.engine.muffler.muffler_elements.iter_mut() {
            let ret = muffler_line.pop();
            muffler_wg_ret.0 += ret.0;
            muffler_wg_ret.1 += ret.1;
            muffler_wg_ret.2 |= ret.2;
        }

        // pop  //
        //////////
        // push //

        for cylinder in self.engine.cylinders.iter_mut() {
            // modulate intake
            cylinder.push(
                self.engine.intake_collector / num_cyl
                    + intake_noise
                        * intake_valve(
                            (self.engine.crankshaft_pos + cylinder.crank_offset).fract(),
                        ),
            );
        }

        self.engine
            .muffler
            .straight_pipe
            .push(self.engine.exhaust_collector, muffler_wg_ret.0);

        self.engine.exhaust_collector += straight_pipe_wg_ret.0;

        let muffler_elements = self.engine.muffler.muffler_elements.len() as f32;

        for muffler_delay_line in self.engine.muffler.muffler_elements.iter_mut() {
            muffler_delay_line.push(straight_pipe_wg_ret.1 / muffler_elements, 0.0);
        }

        engine_vibration = self.engine.engine_vibration_filter.filter(engine_vibration);

        (
            self.engine.intake_collector,
            engine_vibration,
            muffler_wg_ret.1,
            straight_pipe_wg_ret.2 | cylinder_dampened,
        )
    }
}