# rusty-mqtt

A simple MQTT Publisher written in Rust

## Prerequisites

1. Rust, you can install it using [rustup](https://rustup.rs/)
2. [ThingsBoard](https://thingsboard.io/docs/user-guide/install/installation-options/)

## Usage

```
USAGE:
    rusty-mqtt [OPTIONS] --server <SERVER> --username <USER_NAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -n, --devices_number <NUMBER>    Number of devices to spawn [default: 2]
    -p, --port <PORT>                Server's port [default: 1883]
    -s, --server <SERVER>            MQTT server address [default: 0.0.0.0]
    -u, --username <USER_NAME>       Gateway ACCESS_TOKEN
```

### Example

```
$ cargo run -- -s "0.0.0.0" -u "fG9zwdYO83B2DjWyniN2" -n 4
```

## Blog Post

On [Medium](https://medium.com/@LRazovic/mqtt-protocol-using-thingsboard-rust-and-react-9f0434bd206e)

## YouTube Video

[https://www.youtube.com/watch?v=6th-NgDjC1w&feature=youtu.be](https://www.youtube.com/watch?v=6th-NgDjC1w&feature=youtu.be)

## The "Subscriber"

[https://github.com/lrazovic/js-mqtt](https://github.com/lrazovic/js-mqtt)
