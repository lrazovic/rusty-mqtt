use serde::Serialize;

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
    let device = MeteoStation::new(1., 2, 3, 4, 5);
}
