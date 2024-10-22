use anyhow::bail;
use derivative::Derivative;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::hal::adc::{
    AdcContConfig, AdcContDriver, AdcMeasurement, Attenuated, EmptyAdcChannels,
};
use std::convert::TryFrom;

use crate::median::*;
use crate::Config;
use crate::PERIPHERALS;

const VOLTAGE_CHANNEL: usize = 0;

#[derive(Derivative)]
//#[derivative(Default, Copy, Clone, Debug)]
#[derivative(Default)]
struct ChannelBuffers {
    zero_raw: u16,
    reference_rms: f32,
    reference_raw: f32,
    #[derivative(Default(value = "0"))]
    zero_deviation: i32,
    #[derivative(Default(value = "0"))]
    squares_sum: u32,
    #[derivative(Default(value = "0"))]
    count: u16,
    #[derivative(Default(value = "[0; 5]"))]
    median_buf: [u16; 5],
    #[derivative(Default(value = "0"))]
    median_idx: usize,
    #[derivative(Default(value = "false"))]
    median_started: bool,
}

pub struct RmsMeashureResult {
    pub volatge_rms: f32,
    pub current_rms: f32,
    pub frequency: f32,
}

pub fn rms_meashure(app_config: &Config) -> anyhow::Result<RmsMeashureResult> {
    let peripherals = PERIPHERALS.clone();
    let mut peripherals = peripherals.lock();
    let adc_config = AdcContConfig {
        sample_freq: esp_idf_hal::prelude::Hertz(app_config.sps_freq),
        frame_measurements: 16,
        frames_count: 4,
    };

    let adc_channels = EmptyAdcChannels::chain(Attenuated::none(unsafe {
        peripherals.pins.gpio0.clone_unchecked()
    }))
    .chain(Attenuated::none(unsafe {
        peripherals.pins.gpio1.clone_unchecked()
    }));
    let mut adc = AdcContDriver::new(
        unsafe { peripherals.adc1.clone_unchecked() },
        &adc_config,
        adc_channels,
    )?;

    adc.start()?;
    let mut ch = [
        ChannelBuffers {
            zero_raw: app_config.zero_voltage,
            reference_rms: app_config.reference_voltage,
            reference_raw: app_config.reference_voltage_raw,
            ..Default::default()
        },
        ChannelBuffers {
            zero_raw: app_config.zero_current,
            reference_rms: app_config.reference_current,
            reference_raw: app_config.reference_current_raw,
            ..Default::default()
        },
    ];

    let mut is_zero_crossed = false;
    let mut zero_check_block_count: u16 = 1;
    let mut zero_cross_count: u16 = app_config.rms_half_periods + 1;
    let mut last_voltage: u16 = 0;
    let mut max_measurements: i32 =
        (app_config.sps_freq as i32 * app_config.rms_half_periods as i32 * 2)
            / app_config.min_measured_freq as i32
            + 1;

    let mut samples = [AdcMeasurement::default(); 16];
    'reading_loop: loop {
        if let Ok(num_read) = adc.read(&mut samples, 100) {
            for smp in samples.iter().take(num_read) {
                if max_measurements <= 0 {
                    println!("Can't calculate frequency. May be power off?");
                    bail!("Can't calculate frequency. May be power off?");
                }
                let i: usize = usize::try_from(smp.channel())?;
                if i >= ch.len() {
                    println!("Invalid GPIO");
                    bail!("Invalid GPIO");
                } else {
                    if ch[i].median_idx > 4 {
                        ch[i].median_idx = 0;
                        if !ch[i].median_started {
                            ch[i].median_started = true;
                        }
                    }

                    if ch[i].median_started {
                        let median_value = median5(ch[i].median_buf);
                        if i == VOLTAGE_CHANNEL {
                            if zero_cross_count == 0 {
                                break 'reading_loop;
                            }

                            if zero_check_block_count > 0 {
                                zero_check_block_count -= 1;
                            } else if (last_voltage < app_config.zero_voltage
                                && median_value >= app_config.zero_voltage)
                                || (last_voltage >= app_config.zero_voltage
                                    && median_value < app_config.zero_voltage)
                            {
                                zero_check_block_count = (app_config.sps_freq
                                    / app_config.max_measured_freq as u32
                                    / 2
                                    / app_config.rms_half_periods as u32)
                                    as u16;
                                is_zero_crossed = true;
                                zero_cross_count -= 1;
                            }
                            last_voltage = median_value;
                        }

                        if is_zero_crossed {
                            let sq: u32 = if median_value >= ch[i].zero_raw {
                                (median_value - ch[i].zero_raw) as u32
                            } else {
                                (ch[i].zero_raw - median_value) as u32
                            };
                            ch[i].squares_sum += sq * sq;
                            ch[i].zero_deviation += ch[i].zero_raw as i32 - median_value as i32;
                            ch[i].count += 1;
                        }
                    }

                    ch[i].median_buf[ch[i].median_idx] = smp.data();
                    ch[i].median_idx += 1;
                }
                max_measurements -= 1;
            }
        }
    }
    drop(adc);

    let v_rms = ((ch[0].squares_sum as f32 / ch[0].count as f32).sqrt()
        / app_config.reference_voltage_raw
        * app_config.reference_voltage)
        .round();
    let c_rms = (((ch[1].squares_sum as f32 / ch[1].count as f32).sqrt()
        / app_config.reference_current_raw
        * app_config.reference_current
        * 10f32)
        .round()
        // pickups
        - 3f32)
        / 10f32;

    let res = RmsMeashureResult {
        volatge_rms: if v_rms < 55f32 { 0f32 } else { v_rms },
        current_rms: if v_rms < 55f32 || c_rms < 0.5f32 {
            // current deviation
            0f32
        } else {
            c_rms
        },
        frequency: if v_rms < 55f32 {
            0f32
        } else {
            (app_config.sps_freq as f32 / ch[0].count as f32 / 4f32
                * app_config.rms_half_periods as f32
                * 10f32)
                .round()
                / 10f32
        },
    };

    for (i, chan) in ch.iter().enumerate() {
        println!(
            "GPIO{}: freq = {}, RMS = {} (raw: {} deviation: {}) by count {}",
            i,
            (app_config.sps_freq as f32 * app_config.rms_half_periods as f32
                / chan.count as f32
                / 4f32
                * 10f32)
                .round()
                / 10f32,
            ((chan.squares_sum as f32 / chan.count as f32).sqrt() / chan.reference_raw
                * chan.reference_rms
                * 10f32)
                .round()
                / 10f32,
            (chan.squares_sum as f32 / chan.count as f32).sqrt().round(),
            (chan.zero_deviation as f32 / chan.count as f32).round(),
            chan.count
        );
    }

    Ok(res)
}
