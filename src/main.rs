use clap::{App, Arg};
use log::info;
use serde::Serialize;
use std::env;
use std::net::TcpStream;
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
    let mut stream = TcpStream::connect(host).unwrap();
}
