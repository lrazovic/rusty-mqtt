# rusty-mqtt

A bridge between the MQTT broker of TTN and ThingsBoard in Rust built for the Internet of Things 19/20 course during the Engineering in Computer Science Master's Degree. 

## LinkedIn Profile

[Leonardo Razovic](https://www.linkedin.com/in/leonardo-razovic-4b20b1121/)

## Prerequisites

1. Rust, you can install it using [rustup](https://rustup.rs/)
2. [ThingsBoard](https://thingsboard.io/docs/user-guide/install/installation-options/)
3. A [B-L072Z-LRWAN1 LoRa kit](https://www.st.com/en/evaluation-tools/b-l072z-lrwan1.html) or an [IoT-LAB](https://www.iot-lab.info/) Account
4. An account on [The Things Network](https://www.thethingsnetwork.org/)

## Usage

```
USAGE:
    rusty-mqtt [OPTIONS] --TTN-port <RPORT> --server <SERVER> --topic <TOPIC> --username <USER_NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --TTN-port <RPORT>        TTN MQTT server port
    -s, --server <SERVER>         ThingsBoard MQTT server address [default: 0.0.0.0]
    -t, --topic <TOPIC>           TTN topic to subscribe
    -p, --port <TPORT>            ThingsBoard MQTT Server port [default: 1883]
    -r, --TTN-address <TTN>       TTN MQTT server address [default: eu.thethings.network]
    -u, --username <USER_NAME>    ThingsBoard gateway device ACCESS_TOKEN
```
In the `src` folder you need a file `credentials.rs` containing the *App ID* and the *App Access Key* from the **TTN Application Console**. You can modify and rename the `sample_credentials.rs` file as reference.

### Example

```
$ cargo run -- -k 1883 -t "loraiotlab/devices/+/up" -u "8gPybcTugiggd2FVtD0i" -r "eu.thethings.network"
```

## Blog Post

Assignment 1: [The MQTT protocol using ThingsBoard, Rust and React](https://medium.com/@LRazovic/mqtt-protocol-using-thingsboard-rust-and-react-9f0434bd206e)

Assignment 2: [How to setup an Async MQTT transparent bridge in Rust](https://medium.com/@LRazovic/how-to-setup-an-async-mqtt-transparent-bridge-in-rust-4614ad705138)

Assignment 3: [The LoRaWAN communication protocol using RIOT, ThingsBoard and Rust](https://medium.com/@LRazovic/the-lorawan-communication-protocol-using-riot-thingsboard-and-rust-bebe76b20177)

## YouTube Video

Assignment 1: [https://www.youtube.com/watch?v=6th-NgDjC1w&feature=youtu.be](https://www.youtube.com/watch?v=6th-NgDjC1w&feature=youtu.be)

Assignment 2: [https://youtu.be/JiG8LkaZDtQ](https://youtu.be/JiG8LkaZDtQ)

Assignment 3: [https://www.youtube.com/watch?v=bsJNijxUCw0](https://www.youtube.com/watch?v=bsJNijxUCw0)

## The "Subscriber"

[https://github.com/lrazovic/js-mqtt](https://github.com/lrazovic/js-mqtt)
