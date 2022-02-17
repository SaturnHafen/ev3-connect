use confy::load_path;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::io::prelude::*;
use std::net::TcpStream;
use websocket::sync::stream::TlsStream;
use websocket::sync::Client;
use websocket::{ClientBuilder, Message, OwnedMessage};

mod labview;

const CONFIG_PATH: &str = "./config.toml";

#[derive(Serialize, Deserialize, Clone)]
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

fn rejected(response: serde_json::Value) {
    let error = response["Rejected"].to_string();

    println!(
        "Couldn't connect to remote, please inform the server operator. Reason: {}",
        error
    );
}

fn load_config() -> Config {
    let config: Config = load_path(CONFIG_PATH).expect("Couldn't read config file");

    config
}

fn connect_ws(config: &mut Config) -> Client<TlsStream<TcpStream>> {
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

    match &config.ev3 {
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
            println!("Got: {}", payload);

            let resp: serde_json::Value = from_str(&payload).expect("Couldn't parse json...");

            if resp["Control"] != serde_json::Value::Null {
                let ev3 = resp["Control"].to_string();
                config.ev3 = Some(ev3);
            }

            if resp["Rejected"] != serde_json::Value::Null {
                rejected(resp);
                panic!();
            }

            if resp["Queue"] != serde_json::Value::Null {
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
                            let resp: serde_json::Value =
                                from_str(&payload).expect("Couldn't parse json...");

                            if resp["Control"] != serde_json::Value::Null {
                                let ev3 = resp["Control"].to_string();
                                config.ev3 = Some(ev3);

                                // We are now in control!
                                break;
                            }
                        }

                        OwnedMessage::Ping(payload) => {
                            client
                                .send_message(&Message::pong(payload))
                                .expect("Couldn't send pong message");
                        }

                        _ => {
                            debug_assert!(false, "Got strange reply-message from server!");
                        }
                    }
                }
            }
        }
        _ => (),
    }

    client
}

fn main() {
    let mut config = load_config();
    let mut websocket = connect_ws(&mut config);
    let mut ev3_config = labview::Labview::default();

    let name = config.ev3.unwrap();

    ev3_config.name = name;
    ev3_config.spawn_connect_thread();
    let mut labview_connection = ev3_config.connect();

    let mut buf = [0; 65555]; // 65536 is max value of u16, just use a few more for good measure more for length
    let mut response;

    loop {
        let len = labview_connection
            .read(&mut buf)
            .expect("Couldn't read from LabView connection");

        todo!("Look into request filtering (request)");

        websocket
            .send_message(&Message::binary(&buf[..len]))
            .expect("Couldn't write to WebSocket connection...");

        if buf[4].eq(&labview::DIRECT_COMMAND_NO_REPLY)
            || buf[4].eq(&labview::SYSTEM_COMMAND_NO_REPLY)
        {
            // No response expected
            continue;
        }
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
                    todo!("Look into request filtering (response)");
                    labview_connection
                        .write(&payload)
                        .expect("Couldn't write to LabView connection...");

                    // We got an answer!
                    break;
                }
                _ => {
                    // Unexpected message type
                    debug_assert!(false, "Message type not expected!");
                }
            }
        }
    }
}
