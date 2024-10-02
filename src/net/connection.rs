pub mod connection {

    use std::io::Cursor;

    use crate::{net::BUF_LEN, util::*};
    use bytes::BytesMut;
    use log::{debug, error, warn};
    use tokio::{io::AsyncReadExt, net::TcpStream};

    use crate::net::{frame::Frame, Command};

    use super::super::buffer::Buffer;

    pub struct Connection {
        pub stream: TcpStream,
        pub buf: Buffer,
        pub buffer: BytesMut,
    }

    impl Connection {
        pub fn new(socket: TcpStream) -> Connection {
            Connection {
                stream: socket,
                buf: Buffer::new(),
                buffer: BytesMut::with_capacity(64),
            }
        }

        pub async fn read(&mut self) -> Option<Command> {
            // only reading the message, no further validation.
            // this blocks the task until a full message is available
            let frame: Option<Frame> = self.read_frame().await;

            let frame = if let Some(f) = frame {
                f
            } else {
                warn!("error reading message");
                return None;
            };
            let mut buf = Cursor::new(&self.buffer);
            // now we interpret the message
            let cmd = if let Some(cmd) = Frame::parse(&mut buf) {
                cmd
            } else {
                error!("{fg_red}invalid command received: !{fg_reset}");
                return None;
            };

            debug!("{fg_green}received command: {cmd}!{fg_reset}");

            Some(cmd)
            // and execute the command
            // let response = self.exec(cmd);
            // finally sent respond to client
            // if !self.buf.write(&mut self.conn, response).await {
            //     info!("{fg_red}sending command failed!: {fg_reset}");
            // }
        }

        pub async fn read_frame(&mut self) -> Option<Frame> {
            log::trace!("In Buffer: {} (Length: {})", &self.buf, self.buf.len);
            // read the first byte which indicates the length.
            // this value will be discarded and not be part of the read buffer
            let len = self.stream.read_u8().await;
            let len = match len {
                Ok(n) => {
                    if n as usize > BUF_LEN {
                        log::error!("message-length too big!: {}", n);
                        return None;
                    }
                    n
                }
                Err(e) => {
                    log::error!("error at reading message length: {}", e);
                    panic!("EOF when reading frame");
                }
            };

            // read the actual message into the read buffer
            let n = self.stream.read_exact(&mut self.buf[..len as usize]).await;
            match n {
                Ok(0) => {
                    log::info!("remote closed connection!");
                    None
                } // connection closed
                Err(e) => {
                    log::error!("Error at reading TcpStream: {}", e);
                    None
                }
                Ok(n) => Some(Frame { len: n as u8 }),
            }
        }
    }
}
