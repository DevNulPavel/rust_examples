
use serde::{Deserialize, Serialize};
use crate::gen::wave_guide::WaveGuide;


// Глушитель
#[derive(Serialize, Deserialize)]
pub struct Muffler {
    // Длинна трубы
    pub straight_pipe: WaveGuide,
    // Элементы выхлопа
    pub muffler_elements: Vec<WaveGuide>,
}
