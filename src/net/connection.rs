pub mod connection {

    use crate::net::BUF_LEN;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    use crate::net::Command;

    use super::super::buffer::Buffer;

    pub struct Connection {
        pub stream: TcpStream,
        pub buf: Buffer,
    }

    impl Connection {
        pub fn new(socket: TcpStream) -> Connection {
            Connection {
                stream: socket,
                buf: Buffer::new(),
            }
        }
        // reads a message from client into a buffer and parses it to a command
        pub async fn read(&mut self) {
            self.read_frame().await; // TODO: error handling, see also inside read_frame
            log::debug!("frame is ready in buffer!");
        }

        /* raw reading from a stream and writing into it's own buffer */
        pub async fn read_frame(&mut self) -> bool {
            // read the first byte which indicates the length.
            // this value will be discarded and not be part of the read buffer
            let len = self.stream.read_u8().await;
            let len = match len {
                Ok(n) => {
                    if n as usize > BUF_LEN {
                        log::error!("message-length too big!: {}", n);
                        return false;
                    } else {
                        log::trace!("receiving bytes: {}", len.as_ref().unwrap());
                    }
                    n
                }
                Err(e) => {
                    log::error!("error at reading message length: {}", e);
                    panic!("EOF when reading frame");
                }
            };

            // read {len} bytes from stream and write the actual message into our buffer
            let n = self.stream.read_exact(&mut self.buf[..len as usize]).await;
            match n {
                Ok(0) => {
                    log::info!("remote closed connection!");
                    return false;
                } // connection closed
                Err(e) => {
                    log::error!("Error at reading TcpStream: {}", e);
                    return false;
                }
                Ok(n) => {
                    assert!(n == len as usize); // should always be true because of read_exact()
                                                // specification

                    self.buf.len = len as usize;
                    log::trace!("received frame!: {} (Length: {})", &self.buf, self.buf.len);
                    true
                }
            }
        }

        pub async fn write_out(&mut self, data: &[u8]) -> bool {
            self.buf[0] = data.len() as u8;
            self.buf.len = self.buf[0] as usize + 1;
            for (i, val) in data.iter().enumerate() {
                self.buf[i + 1] = *val;
            }

            let r = self
                .stream
                .write_all(&self.buf[..data.len() as usize])
                .await;
            //
            match r {
                Ok(_) => {
                    log::debug!("wrote {} bytes", data.len());
                    true
                }
                Err(e) => {
                    log::error!("Error writing stream: {}", e);
                    false
                }
            }
        }
    }
}
