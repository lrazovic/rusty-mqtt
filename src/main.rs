use clap::{App, Arg};
use log::{error, info, trace};
use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicFilter;
use mqtt::TopicName;
use mqtt::{Decodable, Encodable, QualityOfService};
use std::env;
use std::io::Write;
use std::net;
use std::net::{IpAddr, SocketAddr};
use std::str;
use std::{collections::HashMap, time::Duration};

use futures::join;
use futures::prelude::*;
use tokio::net::TcpStream;
use tokio::prelude::*;
mod credentials;
mod utils;

#[tokio::main]
async fn main() {
    // Loger Initialization
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    // Parse arguments from CLI
    let matches = App::new("rusty-mqtt")
        .author("Leonardo Razovic <lrazovic@gmail.com>")
        .version("0.2")
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
        .get_matches();
    // ThingsBoard server address, default is localhost.
    let server_addr = matches.value_of("SERVER").unwrap();

    // ThingsBoard port address, default is 1883
    let server_port: u16 = matches.value_of("TPORT").unwrap().parse().unwrap();
    let host = format!("{}:{}", server_addr, server_port);

    // ThingsBoard Gateway Device access_token
    let user_name = matches.value_of("USER_NAME").map(|x| x.to_owned()).unwrap();

    let client_id = matches
        .value_of("CLIENT_ID")
        .map(|x| x.to_owned())
        .unwrap_or_else(utils::generate_client_id);

    let topic_name = matches
        .value_of("TOPIC")
        .map(|x| x.to_owned())
        .unwrap_or_else(|| String::from("hello/world"));

    let _ttn_address = matches.value_of("TTN");
    let ttn_port: u16 = matches
        .value_of("RPORT")
        .unwrap()
        .parse()
        .expect("Port is not a number");

    info!("Client identifier {:?}", client_id);

    // TTN MQTT complete address, host + port
    let ttn_addr = match "52.169.76.255".parse::<IpAddr>() {
        Ok(a) => a,
        _ => unreachable!(),
    };
    info!("Connecting to TTN @ {}:{} ... ", ttn_addr, ttn_port);
    let socket_addr = SocketAddr::new(ttn_addr, ttn_port);

    //Opens a TCP connection to TTN.
    let mut ttn_stream = net::TcpStream::connect(socket_addr).expect("Can't connect to TTN");
    info!(
        "Successfully opended a Stream to TTN @ {}:{}",
        ttn_addr, ttn_port
    );
    //Opens a TCP connection to ThingsBoard.
    info!("Connecting to ThingsBoard @ {:?} ... ", host);
    let mut thingsboard_stream =
        net::TcpStream::connect(&host).expect("Can't connect to ThingsBoard");

    // Create and Send an initial MQTT CONNECT packet to TTN.
    let credentials = credentials::get_credentials();
    let mut conn = ConnectPacket::new("MQTT", &client_id);
    conn.set_clean_session(true);
    conn.set_keep_alive(10);
    conn.set_password(Some(credentials.appaccesskey));
    conn.set_user_name(Some(credentials.appid));
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    ttn_stream.write_all(&buf[..]).unwrap();

    // Check if the connection is accepted
    let connack = ConnackPacket::decode(&mut ttn_stream).unwrap();
    trace!("CONNACK {:?}", connack);

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!(
            "Failed to connect to server, return code {:?}",
            connack.connect_return_code()
        );
    }

    info!("Successfully connected to TTN @ {}:{}", ttn_addr, ttn_port);

    // Create and Send an initial MQTT CONNECT packet to Thingsboard.
    let mut conn = ConnectPacket::new("MQTT", client_id);
    conn.set_clean_session(true);
    conn.set_user_name(Some(user_name));
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    thingsboard_stream.write_all(&buf[..]).unwrap();

    // Check if the connection is accepted
    let connack = ConnackPacket::decode(&mut thingsboard_stream).unwrap();
    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!(
            "Failed to connect to server, return code {:?}",
            connack.connect_return_code()
        );
    }
    info!("Successfully connected to Thingsboard @ {}", host);

    // Create a TopicFilter to Subscribe
    let mut channel_filters: Vec<(TopicFilter, QualityOfService)> = Vec::new();
    channel_filters.push((
        match TopicFilter::new(&topic_name) {
            Ok(a) => a,
            _ => unreachable!(),
        },
        QualityOfService::Level0,
    ));

    // Create and send an MQTT SUBSCRIBE packet to TTN
    info!("Subscribed to {:?}", &channel_filters[0].0);
    let sub = SubscribePacket::new(10, channel_filters);
    let mut buf = Vec::new();
    sub.encode(&mut buf).unwrap();
    ttn_stream.write_all(&buf[..]).unwrap();

    loop {
        let packet = match VariablePacket::decode(&mut ttn_stream) {
            Ok(pk) => pk,
            Err(err) => {
                error!("Error in receiving packet {:?}", err);
                continue;
            }
        };
        trace!("PACKET {:?}", packet);

        if let VariablePacket::SubackPacket(ref ack) = packet {
            if ack.packet_identifier() != 10 {
                panic!("SUBACK packet identifier not match");
            }

            info!("Subscribed!");
            break;
        }
    }

    // Connection made, start the async work
    let mut stream = TcpStream::from_std(ttn_stream).unwrap();
    let (mut mqtt_read, mut mqtt_write) = stream.split();

    let ping_time = Duration::new((10) as u64, 0);
    let mut ping_stream = tokio::time::interval(ping_time);

    // PING TTN
    let ping_sender = async move {
        while let Some(_) = ping_stream.next().await {
            info!("Sending PINGREQ to TTN");

            let pingreq_packet = PingreqPacket::new();

            let mut buf = Vec::new();
            pingreq_packet.encode(&mut buf).unwrap();
            mqtt_write.write_all(&buf).await.unwrap();
        }
    };

    // Decode received packets
    let connection_topic = TopicName::new("v1/gateway/connect").unwrap();
    let mut telemetry = HashMap::new();
    let telemtry_topic = TopicName::new("v1/gateway/telemetry").unwrap();
    let mut connected_device = vec![];
    let receiver = async move {
        while let Ok(packet) = VariablePacket::parse(&mut mqtt_read).await {
            trace!("PACKET {:?}", packet);

            match packet {
                VariablePacket::PingrespPacket(..) => {
                    info!("Receiving PINGRESP from TTN ..");
                }
                VariablePacket::PublishPacket(publ) => {
                    let payload = publ.payload();
                    let jsonvalue: serde_json::Value =
                        serde_json::from_slice(&payload).expect("JSON was not well-formatted");
                    info!("RECV JSON Value: {}", &jsonvalue);

                    let device_name = format!("station_{}", &jsonvalue["dev_id"].as_str().unwrap());
                    let payload_fields = &jsonvalue["payload_fields"]["result"];
                    info!("Values: {}", payload_fields);
                    let raw_str_values: Vec<&str> = payload_fields
                        .as_str()
                        .unwrap()
                        .split_ascii_whitespace()
                        .collect();

                    let raw_values: Vec<i16> =
                        raw_str_values.iter().map(|x| x.parse().unwrap()).collect();

                    // Connect Gateway and Device, only if they are not already connected
                    if !connected_device.contains(&device_name) {
                        let device = utils::Device::new(device_name.clone());
                        let message = serde_json::to_string(&device).unwrap();
                        utils::publish(&mut thingsboard_stream, message, connection_topic.clone());
                        connected_device.push(device_name.clone());
                        info!("Gateway and Device {} connected!", device_name);
                    }
                    // Forward the received message to ThingsBoard
                    let temperature: i16 = raw_values[0] - 50;
                    let values = utils::Values::new(
                        temperature,
                        raw_values[1],
                        raw_values[2],
                        raw_values[3],
                        raw_values[4],
                    );
                    let mut vector_values = Vec::new();
                    let sensor_telemetry = utils::generate_telemtry_packet(values);
                    vector_values.push(sensor_telemetry);
                    telemetry.insert(device_name.clone(), vector_values);
                    let serialized_telemetry = serde_json::to_string(&telemetry).unwrap();
                    utils::publish(
                        &mut thingsboard_stream,
                        serialized_telemetry,
                        telemtry_topic.clone(),
                    );
                }

                VariablePacket::ConnectPacket(_) => {}
                VariablePacket::ConnackPacket(_) => {}
                VariablePacket::PubackPacket(_) => {}
                VariablePacket::PubrecPacket(_) => {}
                VariablePacket::PubrelPacket(_) => {}
                VariablePacket::PubcompPacket(_) => {}
                VariablePacket::PingreqPacket(_) => {}
                VariablePacket::SubscribePacket(_) => {}
                VariablePacket::SubackPacket(_) => {}
                VariablePacket::UnsubscribePacket(_) => {}
                VariablePacket::UnsubackPacket(_) => {}
                VariablePacket::DisconnectPacket(_) => {}
            }
        }
    };

    join!(ping_sender, receiver);
}
