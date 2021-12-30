use std::borrow::Cow;
use std::io::prelude::*;
use std::net::TcpStream;
use tungstenite::client::connect;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::protocol::frame::CloseFrame;
use tungstenite::protocol::{Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;

mod labview;

fn ws_connect(ip: &str, port: u16) -> WebSocket<MaybeTlsStream<TcpStream>> {
    println!("Trying to connect to <{}>", ip);

    let (mut knock_connection, _response) = connect(format!("ws://{}:{}", ip, port))
        .expect(format!("Couldn't connect to {}:{}", ip, port).as_str());

    println!("Connection with remote established...");

    knock_connection
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: Cow::Borrowed(&"Knock successful."),
        }))
        .expect("Couldn't close connection...");

    let (connection, _response) = connect(format!("ws://{}:{}", ip, port))
        .expect(format!("Couldn't connect to {}:{}", ip, port).as_str());

    connection
}

fn main() {
    let mut websocket = ws_connect("localhost", 9001);
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
