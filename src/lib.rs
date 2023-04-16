use dht11::Dht11;
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::{
    gpio::{AnyIOPin, IOPin, InputOutput, PinDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use std::{io::{Read, Write, Result}, cell::RefCell};
use std::net::TcpStream;

const SSID: &str = "bssm_guest";
const PASSWORD: &str = "bssm_guest";

pub const HOST: &str = "100.101.113.79";
const PORT: u8 = 80;

// WiFi Driver, dht11 init 함수
pub fn setup() -> (
    EspWifi<'static>,
    Dht11<PinDriver<'static, AnyIOPin, InputOutput>>,
) {
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let dht11_pin = PinDriver::input_output_od(peripherals.pins.gpio16.downgrade()).unwrap();
    let dht11 = Dht11::new(dht11_pin); // DHT11 init

    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        sys_loop,
        Some(nvs)
    ).unwrap();

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration{
        ssid: SSID.into(),
        password: PASSWORD.into(),
        ..Default::default()
    })).unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }
    println!("Should be connected now");

    (wifi_driver, dht11)
}

pub fn connect_server<'a>() -> Result<RefCell<TcpStream>> {
    let mut stream = TcpStream::connect(format!("{}:{}", HOST, PORT))?;
    stream.write(&[1])?;
    let mut buffer = [0; 128];
    stream.read(&mut buffer)?;

    Ok(RefCell::new(stream))
}
