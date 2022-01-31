use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::str::from_utf8;
use std::thread::{sleep, spawn};
use std::time::Duration;

const SERIAL: &str = "0016534c0221";
const PORT: u16 = 5555;
const NAME: &str = "SKLG-01";
const PROTOCOL: &str = "EV3";

pub fn spawn_connect_thread() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("[ERROR] Couldn't bind to adress...");

    socket
        .set_read_timeout(Some(Duration::new(10, 0)))
        .expect("[ERROR] Couldn't set read timeout...");

    socket
        .set_broadcast(true)
        .expect("[ERROR] Couldn't set broadcast flag...");

    let payload = format!(
        "Serial-Number: {}\r\nPort: {}\r\nName: {}\r\nProtocol: {}\r\n",
        SERIAL, PORT, NAME, PROTOCOL
    );
    println!("-------------- Payload --------------");
    println!("{}", payload);
    println!("-------------------------------------");
    println!("Waiting for Lego LabView to connect...");

    spawn(move || {
        loop {
            let send_buf = payload.as_bytes();
            let status = socket.send_to(send_buf, "127.255.255.255:3015"); // broadcast to loopback

            if status.is_ok() {
                let mut recv_buf = [0; 64];

                let result = socket.recv(&mut recv_buf);

                if result.is_err() {
                    // println!("{:?}", result.err());
                    continue;
                }

                let answer_length = result.unwrap();

                let answer = &mut recv_buf[..answer_length];

                println!(
                        "Connection established! (Lego LabView responded with: {}). Finish connecting by clicking on {} in the bottom-right connection panel in Lego LabView!",
                        from_utf8(answer).unwrap(), NAME
                    );
            }
            sleep(Duration::from_secs(5));
        }
    });
}

pub fn connect() -> TcpStream {
    let connection = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], PORT)))
        .expect("[ERROR] Couldn't bind TCP-Listener...");

    let (mut stream, remote) = connection
        .accept()
        .expect("[Error] Couldn't accept TCP-Connection...");

    println!("Connection accepted from {}", remote);

    let mut recv_buf = [0; 64];

    let response = stream
        .read(&mut recv_buf)
        .expect("[Error] Couldn't read from stream...");

    println!("{}", from_utf8(&recv_buf[..response]).unwrap());

    stream
        .write("Accept:EV340\r\n\r\n".as_bytes())
        .expect("[Error] Couldn't write to TCP-Stream...");

    println!("Connection with Lego LabView established!");
    stream
}
