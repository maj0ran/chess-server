use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
};

pub struct Interface {
    listener: TcpListener,
}

impl Interface {
    pub fn new() -> Interface {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

        Interface { listener }
    }

    pub fn wait_for_message(&self) -> (String, String) {
        let stream = self.listener.accept();

        let r = match stream {
            Ok(r) => r,
            Err(_) => todo!("Failed Network Connection"),
        };
        let cmd = self.handle_connection(r.0);
        cmd
    }

    pub fn handle_connection(&self, mut stream: TcpStream) -> (String, String) {
        let mut buf: [u8; 16] = [0; 16];
        let n = stream.peek(&mut buf);

        let src = String::from(buf[0] as char) + &String::from(buf[1] as char);
        let dst = String::from(buf[2] as char) + &String::from(buf[3] as char);

        (src, dst)
    }
}
