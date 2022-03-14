use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::str::from_utf8;
use std::thread::{sleep, spawn};
use std::time::Duration;

pub const DIRECT_COMMAND_NO_REPLY: u8 = 0x80;
pub const SYSTEM_COMMAND_NO_REPLY: u8 = 0x81;

pub struct Labview {
    serial: String,
    port: u16,
    pub name: String,
    protocol: String,
}

impl ::std::default::Default for Labview {
    fn default() -> Self {
        Self {
            serial: "001612345678".to_string(),
            port: 5555,
            name: "EV3".to_string(),
            protocol: "EV3".to_string(),
        }
    }
}

impl Labview {
    pub fn spawn_connect_thread(&mut self) {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("[!] Couldn't bind to adress");

        socket
            .set_read_timeout(Some(Duration::new(10, 0)))
            .expect("[!] Couldn't set read timeout");

        socket
            .set_broadcast(true)
            .expect("[!] Couldn't set broadcast flag");

        let payload = format!(
            "Serial-Number: {}\r\nPort: {}\r\nName: {}\r\nProtocol: {}\r\n",
            self.serial, self.port, self.name, self.protocol
        );
        dbg!("-------------- Payload --------------");
        dbg!("{}", &payload);
        dbg!("-------------------------------------");
        println!("[*] Waiting for Lego LabView to connect...");

        let name = self.name.clone();
        let pl = payload;

        spawn(move || {
            loop {
                let send_buf = pl.as_bytes();
                let status = socket.send_to(send_buf, "127.255.255.255:3015"); // broadcast to loopback

                if status.is_ok() {
                    let mut recv_buf = [0; 64];

                    let result = socket.recv(&mut recv_buf);

                    if result.is_err() {
                        // Probably timeout (no response)
                        // dbg!("{:?}", result.err());
                        continue;
                    }

                    let answer_length = result.unwrap();

                    let answer = &mut recv_buf[..answer_length];

                    println!(
                        "[*] Got a response ({}) from LEGO LabView!",
                        from_utf8(answer).unwrap()
                    );
                    println!(
                            "[*] Finish connecting by clicking on {} in the bottom-right connection panel in Lego LabView!",
                            name
                        );
                }
                sleep(Duration::from_secs(5));
            }
        });
    }

    pub fn connect(self) -> TcpStream {
        let connection = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], self.port)))
            .expect("[!] Couldn't bind TCP-Listener");

        let (mut stream, remote) = connection
            .accept()
            .expect("[!] Couldn't accept TCP-Connection");

        dbg!("[*] Connection accepted from {}", remote);

        let mut recv_buf = [0; 64];

        let response = stream
            .read(&mut recv_buf)
            .expect("[!] Couldn't read from TCP-Stream");

        dbg!("[*] {}", from_utf8(&recv_buf[..response]).unwrap());

        stream
            .write_all("Accept:EV340\r\n\r\n".as_bytes())
            .expect("[!] Couldn't write to TCP-Stream");

        println!("[*] Connection with Lego LabView established! You are now ready to go!");
        stream
    }
}
