use log::info;
use mqtt::packet::{PublishPacket, QoSWithPacketIdentifier};
use mqtt::Encodable;
use mqtt::TopicName;
use serde::Serialize;
use std::io::Write;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Device {
    device: String,
}

impl Device {
    pub fn new(device: String) -> Self {
        Self { device }
    }
}

#[derive(Serialize)]
pub struct Values {
    temperature: i16,
    humidity: i16,
    rain_height: i16,
    wind_direction: i16,
    wind_intensity: i16,
}

impl Values {
    pub fn new(
        temperature: i16,
        humidity: i16,
        wind_direction: i16,
        wind_intensity: i16,
        rain_height: i16,
    ) -> Self {
        Self {
            temperature,
            humidity,
            wind_direction,
            wind_intensity,
            rain_height,
        }
    }
}

#[derive(Serialize)]
pub struct Sensor {
    ts: u128,
    values: Values,
}

impl Sensor {
    pub fn new(ts: u128, values: Values) -> Self {
        Self { ts, values }
    }
}

// Client ID generation using UUID version 4
pub fn generate_client_id() -> String {
    format!("{}", Uuid::new_v4())
}
// Random values generation

pub fn generate_telemtry_packet(values: Values) -> Sensor {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = since_the_epoch.as_millis();
    Sensor::new(in_ms, values)
}

pub fn publish(stream: &mut TcpStream, msg: String, topic: TopicName) {
    // MQTT PUBLISH packet creation
    let packet = PublishPacket::new(
        topic.clone(),
        QoSWithPacketIdentifier::Level1(10),
        msg.clone(),
    );
    // Encode and Write the packet on the TcpStream
    let mut buf = Vec::new();
    packet.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();
    info!("Message: {} sent on Topic: {:?}", msg, topic);
}
