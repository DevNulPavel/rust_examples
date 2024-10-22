use esp_idf_hal::{delay::FreeRtos, peripheral::Peripheral};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::*,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

use crate::PERIPHERALS;

pub fn connect_wifi(wifi_ssid: &str, wifi_psk: &str) -> anyhow::Result<Box<EspWifi<'static>>> {
    use log::info;

    let auth_method = if wifi_psk.is_empty() {
        info!("Wifi password is empty");
        AuthMethod::None
    } else {
        AuthMethod::WPA2Personal
    };

    let _nvs_default_partition: EspNvsPartition<NvsDefault> = EspDefaultNvsPartition::take()?;
    let peripherals = PERIPHERALS.clone();
    let mut peripherals = peripherals.lock();
    let modem = unsafe { peripherals.modem.clone_unchecked() };
    let sysloop = EspSystemEventLoop::take()?;
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;
    wifi.start()?;
    'wifi_loop: loop {
        let ap_infos = wifi.scan()?;
        let ours = ap_infos.into_iter().find(|a| a.ssid == wifi_ssid);
        let channel = if let Some(ours) = ours {
            info!(
                "Found configured access point {} on channel {}",
                wifi_ssid, ours.channel
            );
            Some(ours.channel)
        } else {
            info!(
                "Configured access point {} not found during scanning, delay 10 seconds and retry",
                wifi_ssid
            );
            FreeRtos::delay_ms(10000);
            continue 'wifi_loop;
        };

        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: wifi_ssid
                .try_into()
                .expect("Could not parse the given SSID into WiFi config"),
            password: wifi_psk
                .try_into()
                .expect("Could not parse the given password into WiFi config"),
            channel,
            auth_method,
            ..Default::default()
        }))?;

        info!("Connecting wifi...");
        if wifi.connect() != Ok(()) {
            continue 'wifi_loop;
        }

        info!("Waiting for DHCP lease...");
        if wifi.wait_netif_up() != Ok(()) {
            continue 'wifi_loop;
        }
        info!("Get IP info");
        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
        info!("Wifi DHCP info: {:?}", ip_info);
        break 'wifi_loop Ok(Box::new(esp_wifi));
    }
}
