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
            volume: 0.1_f32,
            samples_per_second,
            engine,
            dc_lp,
            waveguides_dampened: false,
            recording_currently_clipping: false,
        }
    }

    pub(crate) fn generate(&mut self, buf: &mut [f32]) {
        let crankshaft_pos = self.engine.crankshaft_pos;
        let samples_per_second = self.samples_per_second as f32 * 120.0;

        self.recording_currently_clipping = false;
        self.waveguides_dampened = false;

        let mut i = 1.0;
        let mut ii = 0;
        while ii < buf.len() {
            self.engine.crankshaft_pos =
                (crankshaft_pos + i * self.get_rpm() / samples_per_second).fract();
            let samples = self.gen();
            let sample = (samples.0 * self.get_intake_volume()
                + samples.1 * self.get_engine_vibrations_volume()
                + samples.2 * self.get_exhaust_volume())
                * self.get_volume();
            self.waveguides_dampened |= samples.3;

            // reduces dc offset
            buf[ii] = sample - self.dc_lp.filter(sample);

            i += 1.0;
            ii += 1;
        }

        if let Some(recorder) = &mut self.recorder {
            let bufvec = buf.to_vec();
            let mut recording_currently_clipping = false;
            bufvec
                .iter()
                .for_each(|sample| recording_currently_clipping |= sample.abs() > 1.0);
            self.recording_currently_clipping = recording_currently_clipping;

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

    /// generates one sample worth of data
    /// returns  `(intake, engine vibrations, exhaust, waveguides dampened)`
    fn gen(&mut self) -> (f32, f32, f32, bool) {
        let intake_noise = self
            .engine
            .intake_noise_lp
            .filter(self.engine.intake_noise.step())
            * self.engine.intake_noise_factor;

        let mut engine_vibration = 0.0;

        let num_cyl = self.engine.cylinders.len() as f32;

        let last_exhaust_collector = self.engine.exhaust_collector / num_cyl;
        self.engine.exhaust_collector = 0.0;
        self.engine.intake_collector = 0.0;

        let crankshaft_fluctuation_offset = self
            .engine
            .crankshaft_fluctuation_lp
            .filter(self.engine.crankshaft_noise.step());

        let mut cylinder_dampened = false;

        for cylinder in self.engine.cylinders.iter_mut() {
            let (cyl_intake, cyl_exhaust, cyl_vib, dampened) = cylinder.pop(
                self.engine.crankshaft_pos
                    + self.engine.crankshaft_fluctuation * crankshaft_fluctuation_offset,
                last_exhaust_collector,
                self.engine.intake_valve_shift,
                self.engine.exhaust_valve_shift,
            );

            self.engine.intake_collector += cyl_intake;
            self.engine.exhaust_collector += cyl_exhaust;

            engine_vibration += cyl_vib;
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