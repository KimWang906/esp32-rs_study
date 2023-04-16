use esp32_rust::{ setup, HOST, connect_server};
use esp_idf_hal::delay::{Ets, FreeRtos};
use std::{
    io::{Read, Write},
    time::{Duration, Instant},
};

const DEVICE_NAME: &str = "wang9_device";

fn main() {
    esp_idf_sys::link_patches();
    tracing_subscriber::fmt::init();

    let setup_data = setup();
    let wifi = setup_data.0;
    let mut dht11 = setup_data.1;

    loop {
        let mut dht11_delay = Ets;

        println!(
            "IP info: {:?}",
            wifi.sta_netif().get_ip_info().unwrap()
        );

        match dht11.perform_measurement(&mut dht11_delay) {
            Ok(measurement) => {
                let h = measurement.humidity as f32 / 10.0;
                let temp = measurement.temperature as f32 / 10.0;

                println!("temp: {}C, humidity: {}%", temp, h);

                let client = connect_server();
                match client {
                    Ok(stream) => {
                        let mut buffer = [0; 1024];
                        let url =
                            format!("/bssm_2_4/upload.php?did={DEVICE_NAME}&temp={temp}&humi={h}");
                        stream
                            .borrow_mut()
                            .write_all(
                                format!(
                                    "{} {} HTTP/1.1\r\n
                            Host: {}\r\n
                            Connection: close\r\n\r\n",
                                    "GET", url, HOST
                                )
                                .as_bytes(),
                            )
                            .unwrap();
                        dbg!(&stream);
                        let t = Instant::now();
                        let d = Duration::from_millis(10000);
                        let bytes_available = stream.borrow().peek(&mut buffer).unwrap();
                        loop {
                            if bytes_available == 0 {
                                break;
                            }
                            if Instant::now() - t > d {
                                break;
                            }
                        }

                        while bytes_available == 0 {
                            let read_data = stream.borrow_mut().read(&mut buffer).unwrap();
                            dbg!(read_data);
                            println!(
                                "{}",
                                String::from_utf8(buffer[..read_data].to_vec()).unwrap()
                            );
                        }

                        println!("Disconnect");
                        FreeRtos::delay_ms(10000);
                    }
                    Err(e) => println!("{e:?}"),
                }
            }
            Err(e) => println!("{:?}", e),
        }
        FreeRtos::delay_ms(10000);
    }
}