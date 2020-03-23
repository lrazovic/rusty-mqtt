use mqtt::packet::{PublishPacket, QoSWithPacketIdentifier};
use mqtt::Encodable;
use mqtt::TopicName;
use rand::prelude::ThreadRng;
use rand::Rng;
use serde::Serialize;
use std::io::Write;
use std::net::TcpStream;
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
pub struct Sensors {
    temperature: f32,
    humidity: i16,
    rain_height: i16,
    wind_direction: i16,
    wind_intensity: i16,
}

pub fn generate_client_id() -> String {
    format!("{}", Uuid::new_v4())
}

pub fn generate_packet(mut rng: ThreadRng) -> Sensors {
    Sensors {
        humidity: rng.gen_range(0, 100),
        rain_height: rng.gen_range(0, 50),
        temperature: rng.gen_range(-50.0, 50.0),
        wind_direction: rng.gen_range(0, 359),
        wind_intensity: rng.gen_range(0, 100),
    }
}

pub fn publish(stream: &mut TcpStream, msg: String, topic: TopicName) {
    let packet = PublishPacket::new(
        topic.clone(),
        QoSWithPacketIdentifier::Level1(10),
        msg.clone(),
    );
    let mut buf = Vec::new();
    packet.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();
    println!("Message: {} sent on Topic: {:?}", msg, topic);
}
