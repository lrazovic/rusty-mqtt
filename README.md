# rusty-mqtt

An MQTT transparent bridge in Rust built for the Internet of Things 19/20 course during the Engineering in Computer Science Master's Degree. 

## LinkedIn Profile

[Leonardo Razovic](https://www.linkedin.com/in/leonardo-razovic-4b20b1121/)

## Prerequisites

1. Rust, you can install it using [rustup](https://rustup.rs/)
2. [ThingsBoard](https://thingsboard.io/docs/user-guide/install/installation-options/)
3. [RSMB](https://github.com/eclipse/mosquitto.rsmb) (or any other MQTT-SN/MQTT broker with IPv6 support)

## Usage

```
USAGE:
    rusty-mqtt [OPTIONS] --rsmb-port <RPORT> --server <SERVER> --topic <TOPIC> --username <USER_NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --rsmb-port <RPORT>       RSMB MQTT server port
    -r, --rsmb-address <RSMB>     RSMB MQTT server address
    -s, --server <SERVER>         ThingsBoard MQTT server address [default: 0.0.0.0]
    -t, --topic <TOPIC>           RSMB topic to subscribe
    -p, --port <TPORT>            ThingsBoard MQTT Server port [default: 1883]
    -u, --username <USER_NAME>    ThingsBoard gateway device ACCESS_TOKEN
```

### Example

```
$ cargo run -- -k 1888 -t "sensors/values" -u "8gPybcTugiggd2FVtD0i"
```

## Blog Post

Assignment 1: [The MQTT protocol using ThingsBoard, Rust and React](https://medium.com/@LRazovic/mqtt-protocol-using-thingsboard-rust-and-react-9f0434bd206e)

Assignment 2: [How to setup an Async MQTT transparent bridge in Rust]()

## YouTube Video

Assignment 1: [https://www.youtube.com/watch?v=6th-NgDjC1w&feature=youtu.be](https://www.youtube.com/watch?v=6th-NgDjC1w&feature=youtu.be)

Assignment 2:

## The "Subscriber"

[https://github.com/lrazovic/js-mqtt](https://github.com/lrazovic/js-mqtt)
