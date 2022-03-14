use io_bluetooth::bt::{self, BtAddr, BtStream};
use std::iter;

pub const DIRECT_COMMAND_NO_REPLY: u8 = 0x80;
pub const SYSTEM_COMMAND_NO_REPLY: u8 = 0x81;

pub struct EV3 {
    connection: BtStream,
    pub name: String,
}

impl EV3 {
    pub fn connect(ev3: &BtAddr, name: &str) -> EV3 {
        println!("[*] Connecting to {}", ev3);

        let socket = BtStream::connect(iter::once(ev3), bt::BtProtocol::RFCOMM).unwrap();

        match socket.peer_addr() {
            Ok(addr) => println!("[*] Successfully connected to {}.", addr),
            Err(err) => panic!("[!] An error occured while connecting: {:?}", err),
        }

        EV3 {
            connection: socket,
            name: name.to_string(),
        }
    }

    pub fn send_command(&mut self, payload: &[u8]) -> Vec<u8> {
        let _send_count = self
            .connection
            .send(payload)
            .expect("[!] Couldn't write to EV3 connection...");

        let mut recv_buf = [0; 65555];

        if payload[4].eq(&DIRECT_COMMAND_NO_REPLY) || payload[4].eq(&SYSTEM_COMMAND_NO_REPLY) {
            let result: Vec<u8> = Vec::new();
            result
        } else {
            let recv_count = self
                .connection
                .recv(&mut recv_buf)
                .expect("[!] Couldn't read from connection");

            let recv_data = &recv_buf[..recv_count];

            Vec::from(recv_data)
        }
    }
}
