use confy::load_path;
use ev3::EV3;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use tungstenite::client::connect;
use tungstenite::protocol::{Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;

mod ev3;

#[derive(Serialize, Deserialize)]
struct Config {
    remote: String,
    port: u16,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            remote: "localhost".to_string(),
            port: 9000,
        }
    }
}

fn load_config() -> Config {
    let config: Config = load_path("./config.toml").expect("Couldn't read config file");

    config
}

fn connect_ev3() -> EV3 {
    let ev3 = EV3::connect();
    println!("Connected to: {}!", &ev3.name);

    ev3
}

fn connect_ws(remote: &str, port: u16, ev3_name: &String) -> WebSocket<MaybeTlsStream<TcpStream>> {
    println!("Connecting to {}:{}", remote, port);
    let (mut connection, _response) = connect(format!["ws://{}:{}", remote, port])
        .expect(format!["Couldn't connect to remote <{}> on port <{}>", remote, port].as_str());

    connection
        .write_message(Message::Text(format!["{{'id': {}}}", ev3_name.as_str()])) // JSON as specifiied in ev3cconnect README
        .expect("Couldn't queue init message");

    connection
}

fn main() {
    let config = load_config();

    let mut ev3 = connect_ev3();
    let mut websocket = connect_ws(&config.remote, config.port, &ev3.name);

    loop {
        let msg = websocket.read_message().unwrap();

        if msg.is_binary() {
            let buf = &msg.into_data();
            let response = ev3.send_command(buf);
            if buf[4].eq(&0x80) || buf[4].eq(&0x81) {
                // DIRECT_COMMAND_NO_REPLY || SYSTEM_COMMAND_NO_REPLY -> No need to read data from ev3 connection
                continue;
            }

            websocket
                .write_message(Message::Binary(response))
                .expect("Couldn't write response!");
        }
    }
}
