use confy::{load_path, store_path};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::io::prelude::*;
use std::net::TcpStream;
use tungstenite::client::connect;
use tungstenite::protocol::{Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;

mod labview;

#[derive(Serialize, Deserialize)]
struct Config {
    remote: String,
    port: u16,
    ev3: Option<String>,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            remote: "localhost".to_string(),
            port: 9000,
            ev3: None,
        }
    }
}

const CONFIG_PATH: &str = "./config.toml";

fn load_config() -> Config {
    let config: Config = load_path(CONFIG_PATH).expect("Couldn't read config file");

    config
}

fn connect_ws(mut config: Config) -> WebSocket<MaybeTlsStream<TcpStream>> {
    println!("Connecting to {}:{}", config.remote, config.port);
    let (mut connection, _response) = connect(format!["ws://{}:{}", config.remote, config.port])
        .expect(
            format![
                "Couldn't connect to remote <{}> on port <{}>",
                config.remote, config.port
            ]
            .as_str(),
        );

    let message: String;

    match config.ev3 {
        Some(ev3) => message = format!["{{'preferred_ev3': {}}}", ev3.as_str()],
        None => message = "{}".to_string(),
    }
    connection
        .write_message(Message::Text(message)) // JSON as specifiied in ev3cconnect README
        .expect("Couldn't queue init message");

    let response: Message = connection
        .read_message()
        .expect("Couldn't read from Websocket...");

    if response.is_text() {
        let resp: Value = from_str(response.to_text().expect("Couldn't read text..."))
            .expect("Couldn't parse json...");

        if resp["Control"] != Value::Null {
            let ev3 = resp["Control"].to_string();
            config.ev3 = Some(ev3);

            store_path(CONFIG_PATH, &config).expect("Couldn't store config...");
        }
        // store to Config

        if resp["Queue"] != Value::Null {
            println!(
                "--------------------------------------------------------------------------------"
            );
            println!(
                "Awaiting control from ev3. Until then, the ev3 will NOT show up in LEGO LabView!"
            );
            println!(
                "--------------------------------------------------------------------------------"
            );

            loop {
                let response: Message = connection
                    .read_message()
                    .expect("Couldn't read from Websocket...");

                if response.is_text() {
                    let resp: Value = from_str(response.to_text().expect("CouldnÄt read text..."))
                        .expect("Couldn't parse json...");

                    if resp["Control"] != Value::Null {
                        let ev3 = resp["Control"].to_string();
                        config.ev3 = Some(ev3);

                        store_path(CONFIG_PATH, &config).expect("Couldn't store config...");
                    }
                }
            }
        }
    }

    connection
}

fn main() {
    let mut websocket = connect_ws(load_config());
    let mut labview = labview::connect();

    let mut buf = [0; 65555]; // 65536 is max value of u16, just use a few more for good measure more for length
    let mut length;
    let mut response;

    loop {
        length = labview
            .read(&mut buf)
            .expect("Couldn't read from LabView connection...");

        websocket
            .write_message(Message::Binary((&buf[..length]).to_vec()))
            .expect("Couldn't write to WebSocket connection...");

        if buf[4].eq(&0x00) || buf[4].eq(&0x01) {
            // DIRECT_COMMAND_REPLY || SYSTEM_COMMAND_REPLY

            response = websocket
                .read_message()
                .expect("Couldn't read from websocket connection...");

            labview
                .write(&response.into_data())
                .expect("Couldn't write to LabView connection...");
        } else if buf[4].eq(&0x80) || buf[4].eq(&0x81) {
            // DIRECT_COMMAND_NO_REPLY || SYSTEM_COMMAND_NO_REPLY

            continue;
        } else {
            debug_assert!(false, "Got a strange message type!");
        }
    }
}
