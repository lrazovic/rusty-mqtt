# rusty-mqtt

A bridge between the MQTT broker of TTN and ThingsBoard in Rust built for the Internet of Things 19/20 course during the Engineering in Computer Science Master's Degree.

## LinkedIn Profile

[Leonardo Razovic](https://www.linkedin.com/in/leonardo-razovic-4b20b1121/)

## Assignment 1/2/3

### Prerequisites

1. Rust, you can install it using [rustup](https://rustup.rs/)
2. A working instance of [ThingsBoard](https://thingsboard.io/docs/user-guide/install/installation-options/)
3. A [B-L072Z-LRWAN1 LoRa kit](https://www.st.com/en/evaluation-tools/b-l072z-lrwan1.html) or an [IoT-LAB](https://www.iot-lab.info/) Account
4. An account on [The Things Network](https://www.thethingsnetwork.org/)

### Usage

```
USAGE:
    rusty-mqtt [OPTIONS] --TTN-port <RPORT> --server <SERVER> --topic <TOPIC> --username <USER_NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --TTN-port <RPORT>        TTN MQTT server port [default: 1883]
    -s, --server <SERVER>         ThingsBoard MQTT server address [default: 0.0.0.0]
    -t, --topic <TOPIC>           TTN topic to subscribe
    -p, --port <TPORT>            ThingsBoard MQTT Server port [default: 1883]
    -r, --TTN-address <TTN>       TTN MQTT server address [default: eu1.thethings.network]
    -u, --username <USER_NAME>    ThingsBoard gateway device ACCESS_TOKEN
```

In the `src` folder you need a file `credentials.rs` containing the _App ID_ and the _App Access Key_ from the **TTN Application Console**. You can modify and rename the `sample_credentials.rs` file as reference.
In the **TTN Application Console** we must specify a custom decoder function like: 
```js
function Decoder(bytes, port) {
  var result = "";
  for (var byte in bytes){
    result += String.fromCharCode(bytes[byte]);
  } 
  return {"result": result };
}
```

#### Example

```bash
cargo run --release -- -t "#" -u "A1_TEST_TOKEN" -r "eu1.cloud.thethings.network"
```
#### Usefull Links

* The Things Network [MQTT Documentation](https://www.thethingsnetwork.org/docs/applications/mqtt/api/)

## Assignment 4

I developed an HTML5 application using the Generic Sensor API that collects data from the accelerator sensor of the mobile phone.
A User Activity Recognition model is executed both on the device (Edge-based approach) and ThingsBoard (Cloud-based approach).

The application is hosted using [GitHub Pages](https://pages.github.com/) and it's available [here](https://lrazovic.github.io/rusty-mqtt/).
You simply need to enter a ThingsBoard Device [Access Token](https://thingsboard.io/docs/user-guide/ui/devices/) and after pressing the start button data from the accelerometer is sent to ThingsBoard using the Telemetry Upload HTTP API.

## Blog Posts

Assignment 1: [The MQTT protocol using ThingsBoard, Rust and React](https://medium.com/@LRazovic/mqtt-protocol-using-thingsboard-rust-and-react-9f0434bd206e)

Assignment 2: [How to setup an Async MQTT transparent bridge in Rust](https://medium.com/@LRazovic/how-to-setup-an-async-mqtt-transparent-bridge-in-rust-4614ad705138)

Assignment 3: [The LoRaWAN communication protocol using RIOT, ThingsBoard and Rust](https://medium.com/@LRazovic/the-lorawan-communication-protocol-using-riot-thingsboard-and-rust-bebe76b20177)

Assignment 4: [Generic Sensor API, Sensors For The Web!](https://medium.com/@LRazovic/generic-sensor-api-sensors-for-the-web-6eacabe279be)

## YouTube Videos

Assignment 1: [https://www.youtube.com/watch?v=6th-NgDjC1w&feature=youtu.be](https://www.youtube.com/watch?v=6th-NgDjC1w&feature=youtu.be)

Assignment 2: [https://youtu.be/JiG8LkaZDtQ](https://youtu.be/JiG8LkaZDtQ)

Assignment 3: [https://www.youtube.com/watch?v=bsJNijxUCw0](https://www.youtube.com/watch?v=bsJNijxUCw0)

Assignment 4: [https://www.youtube.com/watch?v=d5ZlM878lms](https://www.youtube.com/watch?v=d5ZlM878lms)
