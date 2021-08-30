use futures::join;
use log::{error, info, trace};
use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::{
    packet::{
        ConnackPacket, ConnectPacket, Packet, PingreqPacket, SubscribePacket, VariablePacket,
    },
    Decodable, Encodable, QualityOfService, TopicFilter, TopicName,
};
use std::{collections::HashMap, str, time::Duration};
use std::{env, io::Write};
use std::{net, process};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::utils::parse_argument;

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
    let matches = parse_argument();

    // ThingsBoard server address, default is localhost.
    let server_addr = matches.value_of("SERVER").unwrap();

    // ThingsBoard port address, default is 1883
    let server_port: u16 = matches.value_of("TPORT").unwrap().parse().unwrap();
    let host = format!("{}:{}", server_addr, server_port);

    // ThingsBoard Gateway Device access_token
    let user_name = matches
        .value_of("USER_NAME")
        .map(ToString::to_string)
        .unwrap();

    // Device ID
    let client_id = matches
        .value_of("CLIENT_ID")
        .map_or_else(utils::generate_client_id, ToString::to_string);

    let topic_name = matches
        .value_of("TOPIC")
        .map_or_else(|| String::from("hello/world"), ToString::to_string);

    // TTN server address, default is eu.thethings.network.
    let ttn_address = matches.value_of("TTN").map(ToString::to_string).unwrap();

    // TTN port address
    let ttn_port: u16 = matches
        .value_of("RPORT")
        .unwrap()
        .parse()
        .expect("Port is not a number");

    info!("Client identifier {:?}", client_id);

    let url = format!("{}:{}", ttn_address, ttn_port);
    info!("Connecting to TTN @ {} ... ", url);

    // Opens a TCP connection to TTN.
    let mut ttn_stream = net::TcpStream::connect(&url).expect("Can't connect to TTN");

    // Opens a TCP connection to ThingsBoard.
    info!("Connecting to ThingsBoard @ {:?} ... ", host);
    let thingsboard_connection = match net::TcpStream::connect(&host) {
        Ok(stream) => {
            info!("Successfully connected to ThingsBoard @ {}", ttn_address);
            Ok(stream)
        }
        Err(e) => {
            error!("Can't connect to ThingsBoard. Error: {}", e);
            Err(e)
        }
    };
    let mut thingsboard_stream = thingsboard_connection.unwrap();

    // Create and Send an initial MQTT CONNECT packet to TTN.
    let credentials = credentials::get();
    let mut conn = ConnectPacket::new(&client_id);
    conn.set_password(Some(credentials.appaccesskey.to_string()));
    conn.set_user_name(Some(credentials.appid.to_string()));
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

                    let device_name = format!(
                        "station_{}",
                        &jsonvalue["end_device_ids"]["device_id"].as_str().unwrap()
                    );
                    let payload_fields = &jsonvalue["uplink_message"]["decoded_payload"]["result"];

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
                        let device = utils::Device::new(&device_name);
                        let message = serde_json::to_string(&device).unwrap();
                        utils::publish(&mut thingsboard_stream, &message, &connection_topic);
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
                    telemetry.insert(device_name, vector_values);
                    let serialized_telemetry = serde_json::to_string(&telemetry).unwrap();
                    utils::publish(
                        &mut thingsboard_stream,
                        &serialized_telemetry,
                        &telemtry_topic,
                    );
                }

                _ => {}
            }
        }
    };

    join!(ping_sender, receiver);
}
