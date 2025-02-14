pub mod connection {

    use crate::net::Command;
    use crate::net::BUF_LEN;
    use smol::io::AsyncReadExt;
    use smol::io::AsyncWriteExt;
    use smol::net::TcpStream;

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
            log::debug!("trying to read frame...");
            // read the first byte which indicates the length.
            // this value will be discarded and not be part of the read buffer
            let mut len = [0u8; 1];
            match self.stream.read_exact(&mut len).await {
                Ok(_) => {
                    let len = len[0];
                    if len as usize > BUF_LEN {
                        log::error!("message-length too big!: {} bytes", len);
                        return false;
                    } else {
                        log::trace!("receiving bytes: {}", len);
                    }
                    self.buf[0]
                }
                Err(e) => {
                    log::error!("error at reading message length: {}", e);
                    panic!("EOF when reading frame");
                }
            };

            let len = len[0];
            // read {len} bytes from stream and write the actual message into our buffer
            match self.stream.read_exact(&mut self.buf[..len as usize]).await {
                Err(e) => {
                    log::error!("Error at reading TcpStream: {}", e);
                    return false;
                }
                Ok(_) => {
                    self.buf.len = len as usize;
                    log::trace!("received frame!: {} (Length: {})", &self.buf, self.buf.len);
                }
            };
            true
        }

        pub async fn write_out(&mut self, data: &[u8]) -> bool {
            self.buf[0] = data.len() as u8;
            self.buf.len = self.buf[0] as usize + 1;
            for (i, val) in data.iter().enumerate() {
                self.buf[i + 1] = *val;
            }

            log::debug!("buffer to send: {}", self.buf);

            let r = self
                .stream
                .write_all(&self.buf[..data.len() + 1 as usize])
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
