use clap::{App, Arg};
use log::info;
use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicName;
use mqtt::{Decodable, Encodable};
use rand::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize)]
struct Sensors {
    temperature: f32,
    humidity: i16,
    rain_height: i16,
    wind_direction: i16,
    wind_intensity: i16,
}

#[derive(Serialize)]
struct Device {
    device: String,
}

impl Device {
    fn new(device: String) -> Self {
        Self { device }
    }
}

fn generate_client_id() -> String {
    format!("{}", Uuid::new_v4())
}

fn generate_packet(mut rng: ThreadRng) -> Sensors {
    Sensors {
        humidity: rng.gen_range(0, 100),
        rain_height: rng.gen_range(0, 50),
        temperature: rng.gen_range(-50.0, 50.0),
        wind_direction: rng.gen_range(0, 359),
        wind_intensity: rng.gen_range(0, 100),
    }
}

fn publish(stream: &mut TcpStream, msg: String, topic: TopicName) {
    let packet = PublishPacket::new(
        topic.clone(),
        QoSWithPacketIdentifier::Level1(10),
        msg.clone(),
    );
    let mut buf = Vec::new();
    packet.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();
    info!("Message: {} sent on Topic: {:?}", msg, topic);
}

fn main() {
    // Loger Initialization
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    // Parse arguments from CLI
    let matches = App::new("MQTT")
        .author("Leonardo Razovic <lrazovic@gmail.com>")
        .version("0.1")
        .arg(
            Arg::with_name("SERVER")
                .short("s")
                .long("server")
                .default_value("0.0.0.0")
                .takes_value(true)
                .required(true)
                .help("MQTT server address"),
        )
        .arg(
            Arg::with_name("TOPIC")
                .short("t")
                .long("topic")
                .takes_value(true)
                .required(true)
                .help("Topicr to subscribe"),
        )
        .arg(
            Arg::with_name("USER_NAME")
                .short("u")
                .long("username")
                .required(true)
                .takes_value(true)
                .help("Login user name"),
        )
        .arg(
            Arg::with_name("PORT")
                .short("p")
                .long("port")
                .default_value("1883")
                .takes_value(true)
                .help("Server's port"),
        )
        .arg(
            Arg::with_name("NUMBER")
                .short("n")
                .long("number")
                .default_value("2")
                .takes_value(true)
                .help("Number of devices to spawn"),
        )
        .get_matches();
    let server_addr = matches.value_of("SERVER").unwrap();
    let server_port = matches.value_of("PORT").unwrap();
    let host = format!("{}:{}", server_addr, server_port);
    let topic_name = matches.value_of("TOPIC").map(|x| x.to_owned()).unwrap();
    let client_id = matches
        .value_of("CLIENT_ID")
        .map(|x| x.to_owned())
        .unwrap_or_else(generate_client_id);
    let user_name = matches.value_of("USER_NAME").map(|x| x.to_owned()).unwrap();
    let number: i32 = matches
        .value_of("NUMBER")
        .map(|x| x.to_owned())
        .unwrap()
        .parse()
        .unwrap();

    info!("Connecting to {:?} ... ", host);
    info!("Client identifier {:?}", client_id);

    //Opens a TCP connection to a remote host.
    let mut stream = TcpStream::connect(host.clone()).unwrap();

    // Create and Send an initial MQTT CONNECT packet.
    let mut conn = ConnectPacket::new("MQTT", client_id);
    conn.set_clean_session(true);
    conn.set_user_name(Some(user_name));
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    // Check if the connection is accepted
    let connack = ConnackPacket::decode(&mut stream).unwrap();
    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!(
            "Failed to connect to server, return code {:?}",
            connack.connect_return_code()
        );
    }
    info!("Successfully connected to {:?}", host);

    //Connect Gateway and Devices
    for i in 0..number {
        let device_name = format!("station_{}", i);
        let device = Device::new(device_name);
        let message = serde_json::to_string(&device).unwrap();
        let connection_topic = TopicName::new("v1/Device/connect").unwrap();
        publish(&mut stream, message, connection_topic);
        info!("Gateway and Device {} connected!", i);
    }

    // Create and publish random data on the given Topic
    let rng = thread_rng();
    let topic = TopicName::new(topic_name).unwrap();
    let mut map = HashMap::new();
    loop {
        for i in 0..number {
            let key = format!("station_{}", i);
            let message = generate_packet(rng);
            map.insert(key, message);
        }
        let serialized_message = serde_json::to_string(&map).unwrap();
        publish(&mut stream, serialized_message, topic.clone());
        thread::sleep(Duration::from_secs(5))
    }
}
