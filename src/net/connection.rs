pub mod connection {

    use crate::chessmove::ChessMove;
    use crate::chessmove::ToChessMove;
    use crate::net::Command;
    use crate::net::NewGameParams;
    use crate::net::Parameter;
    use crate::net::BUF_LEN;
    use smol::io::AsyncReadExt;
    use smol::io::AsyncWriteExt;
    use smol::net::TcpStream;

    use super::super::buffer::Buffer;
    use crate::net::*;

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
        pub async fn read(&mut self) -> Option<Command> {
            self.read_frame().await; // TODO: error handling, see also inside read_frame
            self.parse()
        }

        /* raw reading from a stream and writing into it's own buffer */
        pub async fn read_frame(&mut self) -> bool {
            // read the first byte which indicates the length.
            // this value will be discarded and not be part of the read buffer
            let mut len = [0u8; 1];
            match self.stream.read_exact(&mut len).await {
                Ok(_) => {
                    let len = len[0];
                    if len as usize > BUF_LEN {
                        log::error!("message-length too big!: {} bytes", len);
                        return false;
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
        /*
         * parsing the byte-encoded content of the buffer.
         * this will convert the data into a Command struct.
         * The command can then later be executed by the chess server
         */
        fn parse(&self) -> Option<Command> {
            let len = self.buf.len;
            if len == 0 {
                log::warn!("parse: zero-length message");
                return None;
            }

            let content = &self.buf[..len];
            let cmd = content[0];
            let params = &content[1..len];
            let params: Vec<&[u8]> = params.split(|c| *c == b' ' as u8).collect();

            let ret = match cmd {
                opcode::NEW_GAME => {
                    if params.len() != 3 {
                        log::error!("host: invalid number of params received!: {}", params.len());
                        return None;
                    }
                    let mode = params[0].to_param();
                    let time = params[1].to_param();
                    let time_inc = params[2].to_param();
                    let game_params = NewGameParams {
                        mode,
                        time,
                        time_inc,
                    };
                    Some(Command::NewGame(game_params))
                }

                opcode::JOIN_GAME => {
                    if params.len() != 2 {
                        log::error!("join: invalid number of params received!: {}", params.len());
                        return None;
                    }
                    let game_id = params[0].to_param();
                    let side = params[1].to_param();

                    let join_params = JoinGameParams { game_id, side };
                    Some(Command::JoinGame(join_params))
                }
                opcode::MAKE_MOVE => {
                    // ingame Move
                    let x = String::from_utf8(params[0].to_vec()).unwrap();
                    let mov: Option<ChessMove> = x.parse();
                    let mov = match mov {
                        Some(m) => m,
                        None => {
                            log::warn!(
                                "[NetClient] got MOVE opcode but could not parse chess move!"
                            );
                            return None;
                        }
                    };
                    Some(Command::Move(1, mov))
                }
                _ => {
                    log::error!("parse: invalid command");
                    None
                }
            };
            // we got a new message so we clear our read buffer
            // //for i in 0..BUF_LEN {
            //     self[i as usize] = 0;
            // };
            ret
        }
    }
}
