use clap::{App, Arg};
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
pub struct Device<'a> {
    device: &'a str,
}

impl<'a> Device<'a> {
    pub const fn new(device: &'a str) -> Self {
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
    pub const fn new(
        temperature: i16,
        humidity: i16,
        wind_direction: i16,
        wind_intensity: i16,
        rain_height: i16,
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

#[derive(Serialize)]
pub struct Sensor {
    ts: u128,
    values: Values,
}

impl Sensor {
    pub const fn new(ts: u128, values: Values) -> Self {
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

pub fn publish(stream: &mut TcpStream, msg: &str, topic: &TopicName) {
    // MQTT PUBLISH packet creation
    let packet = PublishPacket::new(
        topic.clone(),
        QoSWithPacketIdentifier::Level1(10),
        msg.to_string(),
    );
    // Encode and Write the packet on the TcpStream
    let mut buf = Vec::new();
    packet.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();
    info!("Message: {} sent on Topic: {:?}", msg, topic);
}

pub fn parse_argument() -> clap::ArgMatches<'static> {
    App::new("rusty-mqtt")
        .author("Leonardo Razovic <lrazovic@gmail.com>")
        .version("0.4")
        .arg(
            Arg::with_name("SERVER")
                .short("s")
                .long("server")
                .default_value("0.0.0.0")
                .takes_value(true)
                .required(true)
                .help("ThingsBoard MQTT server address"),
        )
        .arg(
            Arg::with_name("USER_NAME")
                .short("u")
                .long("username")
                .required(true)
                .takes_value(true)
                .help("ThingsBoard gateway device ACCESS_TOKEN"),
        )
        .arg(
            Arg::with_name("TPORT")
                .short("p")
                .long("port")
                .default_value("1883")
                .takes_value(true)
                .help("ThingsBoard MQTT Server port"),
        )
        .arg(
            Arg::with_name("TOPIC")
                .short("t")
                .long("topic")
                .required(true)
                .takes_value(true)
                .help("TTN topic to subscribe"),
        )
        .arg(
            Arg::with_name("RPORT")
                .short("k")
                .long("TTN-port")
                .default_value("1883")
                .required(true)
                .takes_value(true)
                .help("TTN MQTT server port"),
        )
        .arg(
            Arg::with_name("TTN")
                .short("r")
                .long("TTN-address")
                .takes_value(true)
                .default_value("eu.thethings.network")
                .help("TTN MQTT server address"),
        )
        .get_matches()
}
