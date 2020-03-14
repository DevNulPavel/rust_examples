

use serde::{Deserialize, Serialize};
use crate::gen::piston_motion;
use crate::gen::exhaust_valve;
use crate::gen::intake_valve;
use crate::gen::fuel_ignition;
use crate::gen::wave_guide::WaveGuide;

/// Represents one audio cylinder
/// It has two `WaveGuide`s each connected from the cylinder to the exhaust or intake collector
/// ```
/// Labels:                                                     \/ Extractor
///                    b      a            a      b           a    b
/// (Intake Collector) <==|IV|> (Cylinder) <|EV|==> (Exhaust) <====> (Exhaust collector)
///
/// a   b
/// <===>   - WaveGuide with alpha / beta sides => alpha controls the reflectiveness of that side
///
/// |IV|    - Intake valve modulation function for this side of the WaveGuide (alpha)
///
/// |EV|    - Exhaust valve modulation function for this side of the WaveGuide (alpha)
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct Cylinder {
    /// offset of this cylinder's piston crank
    pub crank_offset: f32,
    /// waveguide from the cylinder to the exhaust
    pub exhaust_waveguide: WaveGuide,
    /// waveguide from the cylinder to the intake
    pub intake_waveguide: WaveGuide,
    /// waveguide from the other end of the exhaust WG to the exhaust collector
    pub extractor_waveguide: WaveGuide,
    // waveguide alpha values for when the valves are closed or opened
    pub intake_open_refl: f32,
    pub intake_closed_refl: f32,
    pub exhaust_open_refl: f32,
    pub exhaust_closed_refl: f32,

    pub piston_motion_factor: f32,
    pub ignition_factor: f32,
    /// the time it takes for the fuel to ignite in crank cycles (0.0 - 1.0)
    pub ignition_time: f32,

    // running values
    #[serde(skip)]
    pub cyl_sound: f32,
    #[serde(skip)]
    pub extractor_exhaust: f32,
}

impl Cylinder {
    /// takes in the current exhaust collector pressure
    /// returns (intake, exhaust, piston + ignition, waveguide dampened)
    #[inline]
    pub(in crate::gen) fn pop(
        &mut self,
        crank_pos: f32,
        exhaust_collector: f32,
        intake_valve_shift: f32,
        exhaust_valve_shift: f32,
    ) -> (f32, f32, f32, bool) {
        let crank = (crank_pos + self.crank_offset).fract();

        self.cyl_sound = piston_motion(crank) * self.piston_motion_factor
            + fuel_ignition(crank, self.ignition_time) * self.ignition_factor;

        let ex_valve = exhaust_valve((crank + exhaust_valve_shift).fract());
        let in_valve = intake_valve((crank + intake_valve_shift).fract());

        self.exhaust_waveguide.alpha = self.exhaust_closed_refl
            + (self.exhaust_open_refl - self.exhaust_closed_refl) * ex_valve;
        self.intake_waveguide.alpha =
            self.intake_closed_refl + (self.intake_open_refl - self.intake_closed_refl) * in_valve;

        // the first return value in the tuple is the cylinder-side valve-modulated side of the waveguide (alpha side)
        let ex_wg_ret = self.exhaust_waveguide.pop();
        let in_wg_ret = self.intake_waveguide.pop();

        let extractor_wg_ret = self.extractor_waveguide.pop();
        self.extractor_exhaust = extractor_wg_ret.0;
        self.extractor_waveguide
            .push(ex_wg_ret.1, exhaust_collector);

        //self.cyl_sound += ex_wg_ret.0 + in_wg_ret.0;

        (
            in_wg_ret.1,
            extractor_wg_ret.1,
            self.cyl_sound,
            ex_wg_ret.2 | in_wg_ret.2 | extractor_wg_ret.2,
        )
    }

    /// called after pop
    pub(in crate::gen) fn push(&mut self, intake: f32) {
        let ex_in = (1.0 - self.exhaust_waveguide.alpha.abs()) * self.cyl_sound * 0.5;
        self.exhaust_waveguide.push(ex_in, self.extractor_exhaust);
        let in_in = (1.0 - self.intake_waveguide.alpha.abs()) * self.cyl_sound * 0.5;
        self.intake_waveguide.push(in_in, intake);
    }
}
