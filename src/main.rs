use clap::{App, Arg};
use log::info;
use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::{ConnackPacket, ConnectPacket};
use mqtt::TopicName;
use mqtt::{Decodable, Encodable};
use rand::prelude::thread_rng;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

mod utils;
fn main() {
    // Loger Initialization
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    // Parse arguments from CLI
    let matches = App::new("rusty-mqtt")
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
            Arg::with_name("USER_NAME")
                .short("u")
                .long("username")
                .required(true)
                .takes_value(true)
                .help("Gateway ACCESS_TOKEN"),
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
                .long("devices_number")
                .default_value("2")
                .takes_value(true)
                .help("Number of devices to spawn"),
        )
        .get_matches();
    let server_addr = matches.value_of("SERVER").unwrap();
    let server_port = matches.value_of("PORT").unwrap();
    let host = format!("{}:{}", server_addr, server_port);
    let client_id = matches
        .value_of("CLIENT_ID")
        .map(|x| x.to_owned())
        .unwrap_or_else(utils::generate_client_id);
    let user_name = matches.value_of("USER_NAME").map(|x| x.to_owned()).unwrap();
    let devices_number: i32 = matches
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
    let connection_topic = TopicName::new("v1/gateway/connect").unwrap();
    for i in 0..devices_number {
        let device_name = format!("station_{}", i);
        let device = utils::Device::new(device_name);
        let message = serde_json::to_string(&device).unwrap();
        utils::publish(&mut stream, message, connection_topic.clone());
        info!("Gateway and Device {} connected!", i);
    }

    // Create and publish random data on the given Topic
    let rng = thread_rng();
    let telemtry_topic = TopicName::new("v1/gateway/telemetry").unwrap();
    let attribute_topic = TopicName::new("v1/gateway/attributes").unwrap();
    let mut telemetry = HashMap::new();
    let mut attributes = HashMap::new();
    loop {
        for i in 0..devices_number {
            let mut vector_values = Vec::new();
            let key = format!("station_{}", i);
            let values = utils::generate_packet(rng);
            let sensor_telemetry = utils::generate_telemtry_packet(&values);
            let sensor_attributes = utils::generate_attribute_packet(&values);
            vector_values.push(sensor_telemetry);
            telemetry.insert(key.clone(), vector_values);
            attributes.insert(key, sensor_attributes);
        }
        let serialized_telemetry = serde_json::to_string(&telemetry).unwrap();
        let serialized_attributes = serde_json::to_string(&attributes).unwrap();
        utils::publish(&mut stream, serialized_telemetry, telemtry_topic.clone());
        utils::publish(&mut stream, serialized_attributes, attribute_topic.clone());
        thread::sleep(Duration::from_secs(5))
    }
}
