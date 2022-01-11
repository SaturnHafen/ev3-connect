use regex::Regex;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::str::from_utf8;

pub fn to_hex_string(bytes: &[u8]) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();

    strs.join(":")
}

pub struct EV3 {
    connection: TcpStream,
    pub name: String,
}

impl EV3 {
    pub fn connect() -> EV3 {
        let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 3015)))
            .expect("couldn't bind to address");

        let mut buf = [0; 70];

        println!("Receiving data from ev3...");
        let (mut recv_count, remote) = socket.recv_from(&mut buf).expect("Didn't receive data");

        let mut recv_data = &mut buf[..recv_count];

        let re =
            Regex::new(r"Serial-Number: (\w*)\s\nPort: (\d{1,5})\s\nName: (.*)\s\nProtocol: EV3")
                .unwrap();

        let re_match = re
            .captures(from_utf8(&recv_data).expect("Invalid utf-8 sequence"))
            .unwrap();

        println!("Replying to enable TCP...");
        let send_count = socket
            .send_to(&[0; 10], remote)
            .expect("Couldn't send data");

        if send_count != 10 {
            panic!(
                "Should be able to send packet with size 10, was able to send {}!",
                send_count
            );
        }

        let mut stream = TcpStream::connect(SocketAddr::from((remote.ip(), 5555)))
            .expect("TCP-Connection failed");

        let unlock_payload = format!(
            "GET /target?sn={} VMTP1.0\nProtocol: EV3",
            re_match.get(1).unwrap().as_str()
        );

        let name = re_match.get(3).unwrap().as_str().to_string();

        stream
            .write(unlock_payload.as_bytes())
            .expect("Couldn't send data to connection");

        recv_count = stream
            .read(&mut buf)
            .expect("Couldn't read data from connection");

        recv_data = &mut buf[..recv_count];

        print!(
            "Brick reply: {}",
            from_utf8(&recv_data).expect("Invalid utf-8 sequence")
        );

        println!("Brick should be unlocked!");

        EV3 {
            connection: stream,
            name,
        }
    }

    pub fn send_command(&mut self, payload: &[u8]) -> Vec<u8> {
        let _send_count = self
            .connection
            .write(&payload)
            .expect("Couldn't write to EV3 connection...");

        println!("Send to EV3:");
        println!("        | len | cnt |ty| hd  |op");
        println!("Send: 0x|{}|", to_hex_string(&payload));

        let mut recv_buf = [0; 65555];

        if payload[4].eq(&0x80) || payload[4].eq(&0x81) {
            let result: Vec<u8> = Vec::new();
            result
        } else {
            let recv_count = self
                .connection
                .read(&mut recv_buf)
                .expect("Couldn't read from connection");

            let recv_data = &recv_buf[..recv_count];

            println!("        | len | cnt |rs| pl ");
            println!("Recv: 0x|{}|", to_hex_string(&recv_data));

            Vec::from(recv_data)
        }
    }
}
