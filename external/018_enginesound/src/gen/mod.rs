// https://www.researchgate.net/profile/Stefano_Delle_Monache/publication/280086598_Physically_informed_car_engine_sound_synthesis_for_virtual_and_augmented_environments/links/55a791bc08aea2222c746724/Physically-informed-car-engine-sound-synthesis-for-virtual-and-augmented-environments.pdf?origin=publication_detail



//! ## Generator module ##
//!
//! Basic working principle:
//! Every sample-output generating object (Cylinder, WaveGuide, DelayLine, ..) has to be first `pop`ped,
//! it's output worked upon and then new input samples are `push`ed.
//!

///////////////////////////////////////////////////////////////////////////

mod wave_guide;
mod delay_line;
mod loop_buffer;
mod noise;
mod cylinder;
mod generator;
mod low_pass_filter;
mod engine;
mod muffler;

pub use loop_buffer::LoopBuffer;
pub use low_pass_filter::LowPassFilter;
pub use generator::Generator;
pub use engine::Engine;

///////////////////////////////////////////////////////////////////////////

pub const PI2F: f32 = 2.0 * std::f32::consts::PI;
pub const PI4F: f32 = 4.0 * std::f32::consts::PI;
pub const WAVEGUIDE_MAX_AMP: f32 = 20.0; // at this amplitude, a damping function is applied to fight feedback loops

///////////////////////////////////////////////////////////////////////////

pub fn piston_motion(crank_pos: f32) -> f32 {
    (crank_pos * PI4F).cos()
}

pub fn exhaust_valve(crank_pos: f32) -> f32 {
    if 0.75 < crank_pos && crank_pos < 1.0 {
        -(crank_pos * PI4F).sin()
    } else {
        0.0
    }
}

pub fn intake_valve(crank_pos: f32) -> f32 {
    if 0.0 < crank_pos && crank_pos < 0.25 {
        (crank_pos * PI4F).sin()
    } else {
        0.0
    }
}

pub fn fuel_ignition(crank_pos: f32, ignition_time: f32) -> f32 {
    /*if 0.0 < crank_pos && crank_pos < ignition_time {
        (PI2F * (crank_pos * ignition_time + 0.5)).sin()
    } else {
        0.0
    }*/
    if 0.5 < crank_pos && crank_pos < ignition_time / 2.0 + 0.5 {
        (PI2F * ((crank_pos - 0.5) / ignition_time)).sin()
    } else {
        0.0
    }
}

