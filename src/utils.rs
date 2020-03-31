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
pub struct Values {
    temperature: i16,
    humidity: u8,
    rain_height: u8,
    wind_direction: u8,
    wind_intensity: u8,
}

impl Values {
    pub fn new(
        temperature: i16,
        humidity: u8,
        wind_direction: u8,
        wind_intensity: u8,
        rain_height: u8,
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

pub fn generate_telemtry_packet(values: Values) -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = since_the_epoch.as_millis();
    serde_json::to_string(&Sensor::new(in_ms, values)).unwrap()
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
