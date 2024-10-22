use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::hal::adc::{
    AdcContConfig, AdcContDriver, AdcMeasurement, Attenuated, EmptyAdcChannels,
};

use crate::median::*;
use crate::PERIPHERALS;

const NTC_B_VALUE: f32 = 3950f32;
const NTC_NOMINAL_RESISTENCE: f32 = 10000f32;
const NTC_NOMINAL_TEMPERATURE: f32 = 25f32;
const DIVIDER_RESISTENCE: f32 = 10000f32;

pub fn ntc_meashure() -> anyhow::Result<f32> {
    let peripherals = PERIPHERALS.clone();
    let mut peripherals = peripherals.lock();
    let adc_config = AdcContConfig {
        sample_freq: esp_idf_hal::prelude::Hertz(1000),
        frame_measurements: 5,
        frames_count: 2,
    };

    let adc_channels = EmptyAdcChannels::chain(Attenuated::db11(unsafe {
        peripherals.pins.gpio3.clone_unchecked()
    }));

    let mut adc = AdcContDriver::new(
        unsafe { peripherals.adc1.clone_unchecked() },
        &adc_config,
        adc_channels,
    )?;

    adc.start()?;
    let mut samples = [AdcMeasurement::default(); 5];
    let mut median_buf = [0u16; 5];
    let mut buf_idx = 0usize;
    'reading_loop: loop {
        if let Ok(num_read) = adc.read(&mut samples, 100) {
            for smp in samples.iter().take(num_read) {
                median_buf[buf_idx] = smp.data();
                buf_idx += 1;
                if buf_idx >= 5 {
                    break 'reading_loop;
                }
            }
        }
    }
    drop(adc);
    let raw_res = median5(median_buf);

    let res = ((1f32
        / (((DIVIDER_RESISTENCE / NTC_NOMINAL_RESISTENCE / (4095f32 / raw_res as f32 - 1f32))
            .ln()
            / NTC_B_VALUE)
            + 1f32 / (NTC_NOMINAL_TEMPERATURE + 273.15f32))
        - 273.15f32)
        * 10f32)
        .round()
        / 10f32;
    println!("res: {}, raw_res: {}", res, raw_res);

    Ok(res)
}
