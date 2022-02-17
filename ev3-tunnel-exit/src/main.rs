use confy::load_path;
use ev3::EV3;
use io_bluetooth::bt::BtAddr;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::thread::{self, JoinHandle};
use websocket::sync::stream::TlsStream;
use websocket::sync::Client;
use websocket::{ClientBuilder, Message, OwnedMessage};

mod ev3;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    remote: String,
    port: u16,
    path: String,
    ev3s: Vec<EV3Connection>,
}

#[derive(Serialize, Deserialize, Clone)]
struct EV3Connection {
    nap: u16,
    sap: u32,
    name: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            remote: "localhost".to_string(),
            port: 8800,
            path: "ev3c".to_string(),
            ev3s: vec![EV3Connection::default()],
        }
    }
}

impl ::std::default::Default for EV3Connection {
    fn default() -> Self {
        Self {
            nap: 22,
            sap: 1234567890,
            name: "EV3".to_string(),
        }
    }
}

fn load_config() -> Config {
    let config: Config = load_path("./config.toml").expect("Couldn't read config file");

    config
}

fn connect_ev3(ev3: BtAddr, name: &String) -> EV3 {
    let ev3 = EV3::connect(ev3, name);
    println!("Connected to: {}!", &ev3.name);

    ev3
}

fn connect_ws(config: Config, ev3_name: &String) -> Client<TlsStream<TcpStream>> {
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
    client
        .send_message(&Message::text(format![
            "{{\"id\": \"{}\"}}",
            ev3_name.as_str()
        ])) // JSON as specifiied in ev3cconnect README
        .expect("Couldn't queue init message");

    println!("Connected to remote!");

    client
}

fn main() {
    let config = load_config();

    let ev3_configs = config.ev3s.clone();

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for ev3config in ev3_configs.into_iter() {
        let thread_config = config.clone();

        // start thread for every EV3 in config
        let join_handle: JoinHandle<_> = thread::spawn(move || {
            let mut ev3 = connect_ev3(
                BtAddr::nap_sap(ev3config.nap, ev3config.sap),
                &ev3config.name,
            );
            let mut websocket = connect_ws(thread_config, &ev3config.name);

            loop {
                let msg = websocket.recv_message().unwrap();

                match msg {
                    OwnedMessage::Binary(payload) => {
                        if payload.len() == 0 {
                            // controlling student disconnected
                            continue;
                        }
                        let response = ev3.send_command(&payload);
                        if payload[4].eq(&ev3::DIRECT_COMMAND_NO_REPLY)
                            || payload[4].eq(&ev3::SYSTEM_COMMAND_NO_REPLY)
                        {
                            // no need to read data from ev3 connection
                            continue;
                        }

                        websocket
                            .send_message(&Message::binary(response))
                            .expect("Couldn't write response!");
                    }

                    OwnedMessage::Ping(payload) => {
                        websocket.send_message(&Message::pong(payload)).unwrap();
                    }

                    _ => {
                        // discard all other messages
                    }
                }
            }
        });
        handles.push(join_handle);
    }

    // Join every ev3-controlling-thread to keep the main-thread alive
    for handle in handles.into_iter() {
        handle.join().expect("The thread we joined panicked!");
    }
}
