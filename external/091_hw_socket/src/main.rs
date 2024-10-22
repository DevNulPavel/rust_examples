use embedded_svc::{http::Method, io::Write};
use esp_idf_hal::{delay::FreeRtos, gpio::*, peripheral::Peripheral, peripherals::Peripherals};
use esp_idf_svc::{
    hal::io::EspIOError,
    http::server::{Configuration, EspHttpServer},
};
use lazy_static::lazy_static;
use log::info;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::adc_ntc::ntc_meashure;
use crate::rms::rms_meashure;
use crate::wifi::connect_wifi;

pub mod adc_ntc;
pub mod median;
pub mod rms;
pub mod wifi;

lazy_static! {
    pub static ref PERIPHERALS: Arc<Mutex<Peripherals>> =
        Arc::new(Mutex::new(Peripherals::take().unwrap()));
    pub static ref RELAY: Arc<Mutex<PinDriver<'static, Gpio10, Output>>> = {
        let peripherals = PERIPHERALS.clone();
        let mut peripherals = peripherals.lock();
        let power_relay =
            PinDriver::output(unsafe { peripherals.pins.gpio10.clone_unchecked() }).unwrap();
        Arc::new(Mutex::new(power_relay))
    };
}

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default(100)]
    pub max_measured_freq: u16,
    #[default(25)]
    pub min_measured_freq: u16,
    #[default(2)]
    pub rms_half_periods: u16,
    #[default(40000)]
    pub sps_freq: u32,
    #[default(0)]
    pub voltage_channel: usize,
    #[default(2442)]
    pub zero_voltage: u16,
    #[default(220.)]
    pub reference_voltage: f32,
    #[default(877.0)]
    pub reference_voltage_raw: f32,
    #[default(2619)]
    pub zero_current: u16,
    #[default(2.5)]
    pub reference_current: f32,
    #[default(198.)]
    pub reference_current_raw: f32,
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let app_config = CONFIG;
    let _wifi = connect_wifi(app_config.wifi_ssid, app_config.wifi_psk);
    let mut server = EspHttpServer::new(&Configuration::default())?;

    server.fn_handler(
        "/",
        Method::Get,
        |request| -> core::result::Result<(), EspIOError> {
            let html = smart_plug();
            let mut response = request.into_ok_response()?;
            response.write_all(html.as_bytes())?;
            Ok(())
        },
    )?;

    server.fn_handler(
        "/smart_plug_off",
        Method::Get,
        |request| -> core::result::Result<(), EspIOError> {
            let html = smart_plug_off();
            let mut response = request.into_ok_response()?;
            response.write_all(html.as_bytes())?;
            Ok(())
        },
    )?;

    server.fn_handler(
        "/smart_plug_on",
        Method::Get,
        |request| -> core::result::Result<(), EspIOError> {
            let html = smart_plug_on();
            let mut response = request.into_ok_response()?;
            response.write_all(html.as_bytes())?;
            Ok(())
        },
    )?;

    // Prevent program from exiting
    loop {
        info!("Server awaiting connection");
        FreeRtos::delay_ms(60000);
    }
}

fn smart_plug() -> &'static str {
    let power_relay = RELAY.clone();
    let power_relay = power_relay.lock();
    if power_relay.is_set_high() {
        concat!(
            include_str!("index-0.html"),
            " checked>",
            include_str!("index-1.html")
        )
    } else {
        concat!(
            include_str!("index-0.html"),
            ">",
            include_str!("index-1.html")
        )
    }
}

fn smart_plug_on() -> String {
    let app_config = CONFIG;
    let power_relay = RELAY.clone();
    let mut power_relay = power_relay.lock();
    power_relay.set_low().unwrap();
    let res = rms_meashure(&app_config);
    let temperature = ntc_meashure().unwrap();
    match res {
        Ok(result) => format!(
            "{{\"v\":{},\"f\":{},\"t\":{},\"c\":{},\"p\":{}}}",
            result.volatge_rms,
            result.frequency,
            temperature,
            result.current_rms,
            (result.volatge_rms * result.current_rms).round()
        ),
        Err(_) => format!(
            "{{\"v\":\"{}\",\"f\":\"{}\",\"t\":{},\"c\":\"{}\",\"p\":\"{}\"}}",
            "N/A", "N/A", temperature, "N/A", "N/A"
        ),
    }
}

fn smart_plug_off() -> String {
    let app_config = CONFIG;
    let power_relay = RELAY.clone();
    let mut power_relay = power_relay.lock();
    power_relay.set_high().unwrap();
    let res = rms_meashure(&app_config);
    let temperature = ntc_meashure().unwrap();
    match res {
        Ok(result) => format!(
            "{{\"v\":{},\"f\":{},\"t\":{},\"c\":\"{}\",\"p\":\"{}\"}}",
            result.volatge_rms, result.frequency, temperature, "N/A", "N/A"
        ),
        Err(_) => format!(
            "{{\"v\":\"{}\",\"f\":\"{}\",\"t\":{},\"c\":\"{}\",\"p\":\"{}\"}}",
            "N/A", "N/A", temperature, "N/A", "N/A"
        ),
    }
}
