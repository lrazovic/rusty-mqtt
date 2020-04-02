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
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::{collections::HashMap, time::Duration};

use futures::join;
use futures::prelude::*;
use tokio::net::TcpStream;
use tokio::prelude::*;
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
                .help("ThingsBoard device ACCESS_TOKEN"),
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
                .help("RSMB topic to subscribe"),
        )
        .arg(
            Arg::with_name("RPORT")
                .short("k")
                .long("rsmb-port")
                .required(true)
                .takes_value(true)
                .help("RSMB MQTT server port"),
        )
        .arg(
            Arg::with_name("RSMB")
                .short("r")
                .long("rsmb-address")
                .takes_value(true)
                .help("RSMB MQTT server address"),
        )
        .get_matches();
    // ThingsBoard server address, default is localhost.
    let server_addr = matches.value_of("SERVER").unwrap();

    // ThingsBoard port address, default is 1883
    let server_port: u16 = matches.value_of("TPORT").unwrap().parse().unwrap();
    let host = format!("{}:{}", server_addr, server_port);

    let client_id = matches
        .value_of("CLIENT_ID")
        .map(|x| x.to_owned())
        .unwrap_or_else(utils::generate_client_id);

    let topic_name = matches
        .value_of("TOPIC")
        .map(|x| x.to_owned())
        .unwrap_or_else(|| String::from("hello/world"));

    let rsmb_address = matches.value_of("RSMB");
    let rsmb_port: u16 = matches
        .value_of("RPORT")
        .unwrap()
        .parse()
        .expect("Port is not a number");

    // ThingsBoard Gateway Device access_token
    let user_name = matches.value_of("USER_NAME").map(|x| x.to_owned()).unwrap();

    info!("Connecting to {:?} ... ", host);
    info!("Client identifier {:?}", client_id);

    // RSMB address, using IPv6, default is localhost
    let ipv6_addr = match rsmb_address {
        None => IpAddr::V6(Ipv6Addr::LOCALHOST),
        Some(x) => {
            let ip_addr: Ipv6Addr = x.parse().expect("Invalid Address");
            IpAddr::V6(ip_addr)
        }
    };

    // RSMB MQTT complete address, host + port
    let socket_addr = SocketAddr::new(ipv6_addr, rsmb_port);

    //Opens a TCP connection to RSMB.
    let mut rsmb_stream = net::TcpStream::connect(socket_addr).expect("Can't connect to RSMB");

    //Opens a TCP connection to ThingsBoard.
    let mut thingsboard_stream =
        net::TcpStream::connect(&host).expect("Can't connect to ThingsBoard");

    // Create and Send an initial MQTT CONNECT packet to RSMB.
    let mut conn = ConnectPacket::new("MQTT", &client_id);
    conn.set_clean_session(true);
    conn.set_keep_alive(10);
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    rsmb_stream.write_all(&buf[..]).unwrap();

    // Check if the connection is accepted
    let connack = ConnackPacket::decode(&mut rsmb_stream).unwrap();
    trace!("CONNACK {:?}", connack);

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!(
            "Failed to connect to server, return code {:?}",
            connack.connect_return_code()
        );
    }

    info!(
        "Successfully connected to RSMB @ [{}]:{}",
        ipv6_addr, rsmb_port
    );

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

    // Create and send an MQTT SUBSCRIBE packet to RSMB
    let sub = SubscribePacket::new(10, channel_filters);
    let mut buf = Vec::new();
    sub.encode(&mut buf).unwrap();
    rsmb_stream.write_all(&buf[..]).unwrap();

    loop {
        let packet = match VariablePacket::decode(&mut rsmb_stream) {
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
    let mut stream = TcpStream::from_std(rsmb_stream).unwrap();
    let (mut mqtt_read, mut mqtt_write) = stream.split();

    let ping_time = Duration::new((30 / 2) as u64, 0);
    let mut ping_stream = tokio::time::interval(ping_time);

    // PING RSMB
    let ping_sender = async move {
        while let Some(_) = ping_stream.next().await {
            info!("Sending PINGREQ to RSMB");

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
                    info!("Receiving PINGRESP from RSMB ..");
                }
                VariablePacket::PublishPacket(publ) => {
                    let msg = publ.payload();
                    let device_name = format!("station_{}", msg[0]);

                    // Connect Gateway and Device, only if they are not already connected
                    if !connected_device.contains(&device_name) {
                        let device = utils::Device::new(device_name.clone());
                        let message = serde_json::to_string(&device).unwrap();
                        utils::publish(&mut thingsboard_stream, message, connection_topic.clone());
                        connected_device.push(device_name.clone());
                        info!("Gateway and Device {} connected!", msg[0]);
                    }
                    // Forward the received message to ThingsBoard
                    let temperature: i16 = (msg[1]) as i16 - 50;
                    let values = utils::Values::new(temperature, msg[2], msg[3], msg[4], msg[5]);
                    info!("RECV on Topic : {:?}", msg);
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
                _ => {
                    info!("Receiving UNHANDLED PACKET from broker ..");
                }
            }
        }
    };

    join!(ping_sender, receiver);
}
