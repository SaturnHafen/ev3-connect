use ev3::EV3;
use std::net::TcpListener;
use tungstenite::{accept, Message};

mod ev3;

fn connect_ev3() -> EV3 {
    let ev3 = EV3::connect();

    ev3
}

fn main() {
    let mut ev3 = connect_ev3();

    let server = TcpListener::bind("0.0.0.0:9001").unwrap();

    println!("Awaiting remote to connect...");

    for stream in server.incoming() {
        let mut websocket = accept(stream.unwrap()).unwrap();
        println!("Remote connected. Now entering loop...");
        loop {
            let msg = websocket.read_message().unwrap();

            // TODO: Instead of using this as server, use external server and just connect here.

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
}
