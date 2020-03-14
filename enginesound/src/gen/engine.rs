use serde::{Deserialize, Serialize};
use crate::gen::low_pass_filter::LowPassFilter;
use crate::gen::cylinder::Cylinder;
use crate::gen::noise::Noise;
use crate::gen::muffler::Muffler;


#[derive(Serialize, Deserialize)]
pub struct Engine {
    pub rpm: f32,
    pub intake_volume: f32,
    pub exhaust_volume: f32,
    pub engine_vibrations_volume: f32,

    pub cylinders: Vec<Cylinder>,
    #[serde(skip)]
    pub intake_noise: Noise,
    pub intake_noise_factor: f32,
    pub intake_noise_lp: LowPassFilter,
    pub engine_vibration_filter: LowPassFilter,
    pub muffler: Muffler,
    /// valve timing -0.5 - 0.5
    pub intake_valve_shift: f32,
    /// valve timing -0.5 - 0.5
    pub exhaust_valve_shift: f32,
    pub crankshaft_fluctuation: f32,
    pub crankshaft_fluctuation_lp: LowPassFilter,
    #[serde(skip)]
    pub crankshaft_noise: Noise,
    // running values
    /// crankshaft position, 0.0-1.0
    #[serde(skip)]
    pub crankshaft_pos: f32,
    #[serde(skip)]
    pub exhaust_collector: f32,
    #[serde(skip)]
    pub intake_collector: f32,
}
