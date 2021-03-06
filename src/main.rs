use clap::{App, Arg};
use futures::join;
use log::{error, info, trace};
use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::{packet::*, Decodable, Encodable, QualityOfService, TopicFilter, TopicName};
use std::{collections::HashMap, str, time::Duration};
use std::{env, io::Write};
use std::{net, process};
use tokio::{io::AsyncWriteExt, net::TcpStream};

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

    // Device ID
    let client_id = matches
        .value_of("CLIENT_ID")
        .map(|x| x.to_owned())
        .unwrap_or_else(utils::generate_client_id);

    let topic_name = matches
        .value_of("TOPIC")
        .map(|x| x.to_owned())
        .unwrap_or_else(|| String::from("hello/world"));

    // TTN server address, default is eu.thethings.network.
    let ttn_address = matches.value_of("TTN").map(|x| x.to_owned()).unwrap();

    // TTN port address
    let ttn_port: u16 = matches
        .value_of("RPORT")
        .unwrap()
        .parse()
        .expect("Port is not a number");

    info!("Client identifier {:?}", client_id);

    let url = format!("{}:{}", ttn_address, ttn_port);
    info!("Connecting to TTN @ {} ... ", url);

    //Opens a TCP connection to TTN.
    let mut ttn_stream = net::TcpStream::connect(&url).expect("Can't connect to TTN");

    //Opens a TCP connection to ThingsBoard.
    info!("Connecting to ThingsBoard @ {:?} ... ", host);
    let mut thingsboard_stream =
        net::TcpStream::connect(&host).expect("Can't connect to ThingsBoard");

    // Create and Send an initial MQTT CONNECT packet to TTN.
    let credentials = credentials::get_credentials();
    let mut conn = ConnectPacket::new(&client_id);
    conn.set_password(Some(credentials.appaccesskey));
    conn.set_user_name(Some(credentials.appid));
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    ttn_stream.write_all(&buf[..]).unwrap();

    // Check if the connection is accepted
    let connack = ConnackPacket::decode(&mut ttn_stream).unwrap();

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        error!(
            "Failed to connect to TTN. Error: {:?}",
            connack.connect_return_code()
        );
        process::exit(1);
    }

    info!("Successfully connected to TTN @ {}", url);

    // Create and send an initial MQTT CONNECT packet to Thingsboard.
    let mut conn = ConnectPacket::new(client_id);
    conn.set_clean_session(true);
    conn.set_user_name(Some(user_name));
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    thingsboard_stream.write_all(&buf[..]).unwrap();

    // Check if the connection is accepted
    let connack = ConnackPacket::decode(&mut thingsboard_stream).unwrap();
    match connack.connect_return_code() {
        ConnectReturnCode::ConnectionAccepted => {
            info!("Successfully connected to Thingsboard @ {}", host);
        }
        _ => {
            error!(
                "Failed to connect to Thingsboard. Error: {:?}",
                connack.connect_return_code()
            );
            process::exit(1);
        }
    }

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
                error!("SUBACK packet identifier not match");
                process::exit(1);
            }
            info!("Subscribed!");
            break;
        }
    }

    // Connection made, start the async work
    let mut stream = TcpStream::from_std(ttn_stream).unwrap();
    let (mut mqtt_read, mut mqtt_write) = stream.split();

    let ping_time = Duration::new(120, 0);
    let mut ping_stream = tokio::time::interval(ping_time);

    // PING TTN
    let ping_sender = async move {
        loop {
            ping_stream.tick().await;
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
                    // Take the payload, in bytes
                    let payload = publ.payload();

                    // From bytes to JSON
                    let jsonvalue: serde_json::Value =
                        serde_json::from_slice(&payload).expect("JSON was not well-formatted");

                    let device_name = format!("station_{}", &jsonvalue["dev_id"].as_str().unwrap());
                    let payload_fields = &jsonvalue["payload_fields"]["result"];

                    // Works with a payload decoder format on TTN like
                    // function Decoder(bytes, port) {
                    //    var result = "";
                    //    for (var byte in bytes){
                    //      result += String.fromCharCode(bytes[byte]);
                    //    } 
                    //    return {"result": result };
                    // }

                    // A test payload, in bytes, can be "38 30 20 33 30 20 32 31 33 20 33 35 20 34 35"

                    // Get the 5 values from the JSON
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
                    let temperature = raw_values[0] - 50;
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

                _ => {}
            }
        }
    };

    join!(ping_sender, receiver);
}
