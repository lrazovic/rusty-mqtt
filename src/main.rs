use serde::Serialize;
use std::env;
use log::info;

#[derive(Serialize)]
struct MeteoStation {
    temperature: f32,
    humidity: i16,
    rain_height: i16,
    wind_direction: i16,
    wind_intensity: i16,
}

impl MeteoStation {
    fn new(
        temperature: f32,
        humidity: i16,
        rain_height: i16,
        wind_direction: i16,
        wind_intensity: i16,
    ) -> Self {
        Self {
            temperature,
            humidity,
            rain_height,
            wind_direction,
            wind_intensity,
        }
    }
}

fn main() {
    // Loger Initialization
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    let device = MeteoStation::new(1., 2, 3, 4, 5);
}
