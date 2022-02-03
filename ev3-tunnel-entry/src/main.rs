use confy::{load_path, store_path};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::io::prelude::*;
use std::net::TcpStream;
use websocket::sync::stream::TlsStream;
use websocket::sync::Client;
use websocket::{ClientBuilder, Message, OwnedMessage};

mod labview;

#[derive(Serialize, Deserialize)]
struct Config {
    remote: String,
    port: u16,
    path: String,
    ev3: Option<String>,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            remote: "localhost".to_string(),
            port: 9000,
            path: "ev3c".to_string(),
            ev3: None,
        }
    }
}

const CONFIG_PATH: &str = "./config.toml";

fn load_config() -> Config {
    let config: Config = load_path(CONFIG_PATH).expect("Couldn't read config file");

    config
}

fn connect_ws(mut config: Config) -> Client<TlsStream<TcpStream>> {
    println!(
        "Connecting to {}:{}/{}",
        config.remote, config.port, config.path
    );
    let mut client = ClientBuilder::new(
        format!["wss://{}:{}/{}", config.remote, config.port, config.path].as_str(),
    )
    .unwrap()
    .connect_secure(None)
    .unwrap();

    let message: String;

    match config.ev3 {
        Some(ev3) => message = format!["{{\"preferred_ev3\": \"{}\"}}", ev3.as_str()],
        None => message = "{}".to_string(),
    }

    println!("Sending: {}", message);

    client
        .send_message(&Message::text(message)) // JSON as specifiied in ev3cconnect README
        .expect("Couldn't queue init message");

    let response = client
        .recv_message()
        .expect("Couldn't read from Websocket...");

    match response {
        OwnedMessage::Text(payload) => {
            println!("Got: {:?}", payload);

            let resp: Value = from_str(&payload).expect("Couldn't parse json...");

            if resp["Control"] != Value::Null {
                let ev3 = resp["Control"].to_string();
                config.ev3 = Some(ev3);

                store_path(CONFIG_PATH, &config).expect("Couldn't store config...");
            }

            if resp["Rejected"] != Value::Null {
                let error = resp["Rejected"].to_string();

                println!(
                    "Couldn't connect to remote, please inform the server operator. Reason: {}",
                    error
                );
                panic!();
            }

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
                    let response = client
                        .recv_message()
                        .expect("Couldn't read from Websocket...");

                    match response {
                        OwnedMessage::Text(payload) => {
                            let resp: Value = from_str(&payload).expect("Couldn't parse json...");

                            if resp["Control"] != Value::Null {
                                let ev3 = resp["Control"].to_string();
                                config.ev3 = Some(ev3);

                                store_path(CONFIG_PATH, &config).expect("Couldn't store config...");
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => (),
    }

    client
}

fn main() {
    let mut websocket = connect_ws(load_config());
    labview::spawn_connect_thread();
    let mut labview_connection = labview::connect();

    let mut buf = [0; 65555]; // 65536 is max value of u16, just use a few more for good measure more for length
    let mut response;

    loop {
        let len = labview_connection
            .read(&mut buf)
            .expect("Couldn't read from LabView connection");

        websocket
            .send_message(&Message::binary(&buf[..len]))
            .expect("Couldn't write to WebSocket connection...");

        if buf[4].eq(&0x00) || buf[4].eq(&0x01) {
            // DIRECT_COMMAND_REPLY || SYSTEM_COMMAND_REPLY
            loop {
                response = websocket
                    .recv_message()
                    .expect("Couldn't read from websocket connection...");

                match response {
                    OwnedMessage::Ping(payload) => {
                        websocket
                            .send_message(&Message::pong(payload))
                            .expect("Couldn't send pong reply");
                    }

                    OwnedMessage::Binary(payload) => {
                        if (payload.len() - 2) as u16
                            != (((payload[1] as u16) << 8) | payload[0] as u16)
                        {
                            panic!(
                                "Expected size does not match received size! (Expected: {}, Received: {})",
                                (((payload[0] as u16) << 8) | payload[1] as u16),
                                payload.len() - 2
                            );
                        }

                        labview_connection
                            .write(&payload)
                            .expect("Couldn't write to LabView connection...");

                        break;
                    }
                    _ => {}
                }
            }
        } else if buf[4].eq(&0x80) || buf[4].eq(&0x81) {
            // DIRECT_COMMAND_NO_REPLY || SYSTEM_COMMAND_NO_REPLY
            continue;
        } else {
            debug_assert!(false, "Got a strange message type!");
        }
    }
}
