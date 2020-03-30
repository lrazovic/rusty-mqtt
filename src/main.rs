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
// use std::net::UdpSocket;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str;
use std::time::Duration;

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
                .help("MQTT server address"),
        )
        .arg(
            Arg::with_name("USER_NAME")
                .short("u")
                .long("username")
                .default_value("dyGTnPixbX1WiOsP92DT")
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
        .get_matches();
    let server_addr = matches.value_of("SERVER").unwrap();
    let server_port: u16 = matches.value_of("PORT").unwrap().parse().unwrap();
    let host = format!("{}:{}", server_addr, server_port);
    let client_id = matches
        .value_of("CLIENT_ID")
        .map(|x| x.to_owned())
        .unwrap_or_else(utils::generate_client_id);
    let user_name = matches.value_of("USER_NAME").map(|x| x.to_owned()).unwrap();

    info!("Connecting to {:?} ... ", host);
    info!("Client identifier {:?}", client_id);
    let ipv6_addr = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let socket_addr = SocketAddr::new(ipv6_addr, 1888);

    //Opens a TCP connection to RSMB.
    let mut rsmb_stream = net::TcpStream::connect(socket_addr).unwrap();

    //Opens a TCP connection to ThingsBoard.
    let mut thingsboard_stream = net::TcpStream::connect(&host).unwrap();

    // Create and Send an initial MQTT CONNECT packet to RSMB.
    let mut conn = ConnectPacket::new("MQTT", client_id.clone());
    conn.set_clean_session(true);
    conn.set_keep_alive(10);
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    rsmb_stream.write_all(&buf[..]).unwrap();

    let connack = ConnackPacket::decode(&mut rsmb_stream).unwrap();
    trace!("CONNACK {:?}", connack);

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!(
            "Failed to connect to server, return code {:?}",
            connack.connect_return_code()
        );
    }

    info!("Successfully connected to RSMB @ [{}]:{}", ipv6_addr, 1888);

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

    let mut channel_filters: Vec<(TopicFilter, QualityOfService)> = Vec::new();
    channel_filters.push((
        TopicFilter::new("hello/world").unwrap(),
        QualityOfService::Level0,
    ));
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

    // connection made, start the async work
    let mut stream = TcpStream::from_std(rsmb_stream).unwrap();
    let (mut mqtt_read, mut mqtt_write) = stream.split();

    let ping_time = Duration::new((10 / 2) as u64, 0);
    let mut ping_stream = tokio::time::interval(ping_time);

    let ping_sender = async move {
        while let Some(_) = ping_stream.next().await {
            info!("Sending PINGREQ to broker");

            let pingreq_packet = PingreqPacket::new();

            let mut buf = Vec::new();
            pingreq_packet.encode(&mut buf).unwrap();
            mqtt_write.write_all(&buf).await.unwrap();
        }
    };
    let receiver = async move {
        while let Ok(packet) = VariablePacket::parse(&mut mqtt_read).await {
            trace!("PACKET {:?}", packet);

            match packet {
                VariablePacket::PingrespPacket(..) => {
                    info!("Receiving PINGRESP from broker ..");
                }
                VariablePacket::PublishPacket(publ) => {
                    /*
                    let msg = match str::from_utf8(&publ.payload_ref()[..]) {
                        Ok(msg) => msg,
                        Err(err) => {
                            error!("Failed to decode publish message {:?}", err);
                            continue;
                        }
                    };
                    */
                    let msg = publ.payload();
                    info!("RECV on Topic : {:?}", msg);
                    //let telemtry_topic = TopicName::new("v1/devices/me/telemetry").unwrap();
                    //let value = format!("{{\"temperature\": \"{}\"}}", msg);
                    //utils::publish(&mut thingsboard_stream, value, telemtry_topic.clone());
                }
                _ => {}
            }
        }
    };

    join!(ping_sender, receiver);
}
