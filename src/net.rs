use crate::util::*;
use core::fmt;
use std::ops::{self, RangeTo};

#[allow(unused)]
use log::{debug, error, info, trace, warn};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::game::Game;
use crate::tile::ToChessMove;
const NEW_GAME: u8 = 1;
const JOIN_GAME: u8 = 2;
const SET_NAME: u8 = 3;

const BUF_LEN: usize = 64;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct NewGame {
    mode: String,
    hoster_side: PlayerSideRequest,
    name: String,
}
impl NewGame {
    pub fn new(mode: String, side: PlayerSideRequest) -> NewGame {
        NewGame {
            mode,
            hoster_side: side,
            name: "test123".to_string(),
        }
    }
}

struct Buffer {
    buf: [u8; BUF_LEN],
    len: usize,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            buf: [0; BUF_LEN],
            len: 0,
        }
    }
}
impl ops::Index<usize> for Buffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
    }
}
impl ops::Index<RangeTo<usize>> for Buffer {
    type Output = [u8];

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self.buf[index]
    }
}
impl ops::IndexMut<usize> for Buffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buf[index]
    }
}
impl ops::IndexMut<RangeTo<usize>> for Buffer {
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
        &mut self.buf[index]
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf_str = style_bold.to_string();
        for i in &self.buf[..self.len] {
            let col = match i {
                b' ' => fg_green,
                _ if i < &32 => fg_yellow,
                _ => fg_blue,
            };
            buf_str = buf_str + &format!("{col}[{i}]");
        }
        buf_str = buf_str + fg_reset + style_reset;
        write!(f, "{}", buf_str)
    }
}
#[derive(Debug)]
#[repr(u8)]
enum Command {
    NewGame(NewGame),
    JoinGame(String),
    Nickname(String),
    Move(String),
    Invalid = 0xFF,
}

#[repr(u8)]
#[derive(Debug)]
enum PlayerSideRequest {
    Black = 0,
    White = 1,
    Random = 2,
}

impl TryFrom<u8> for PlayerSideRequest {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            x if x == PlayerSideRequest::White as u8 => Ok(PlayerSideRequest::White),
            x if x == PlayerSideRequest::Black as u8 => Ok(PlayerSideRequest::Black),
            x if x == PlayerSideRequest::Random as u8 => Ok(PlayerSideRequest::Random),
            _ => Err(()),
        }
    }
}

// A token after a command is a Parameter.
// Parameters can be different types so we have
// to define some conversions.
trait Parameter<T> {
    fn to_val(&self) -> T;
}
impl Parameter<String> for &[u8] {
    fn to_val(&self) -> String {
        String::from_utf8_lossy(self).to_string()
    }
}
impl Parameter<u8> for &[u8] {
    fn to_val(&self) -> u8 {
        self[0]
    }
}

// our read buffer can be interpreted as a message.
// Parsing it will result in a Command.
trait Message {
    fn parse(&self) -> Option<Command>;
}
impl Message for Buffer {
    fn parse(&self) -> Option<Command> {
        let tokens: Vec<&[u8]> = self[..self.len].split(|s| &b' ' == s).collect();
        let cmd = tokens[0];
        let params = &tokens[1..];

        match cmd[0] {
            NEW_GAME => {
                if params.len() != 2 {
                    error!("invalid number of params received!: {}", params.len());
                    return None;
                }
                let mode = params[0].to_val();
                let side: u8 = params[1].to_val();
                let side = PlayerSideRequest::try_from(side);
                let side = match side {
                    Ok(s) => s,
                    Err(_) => {
                        warn!("invalid Side chosen! default to random");
                        PlayerSideRequest::Random
                    }
                };
                let new_game = NewGame::new(mode, side);
                Some(Command::NewGame(new_game))
            }

            JOIN_GAME => {
                let game_name = params[0].to_val();
                Some(Command::JoinGame(game_name))
            }
            SET_NAME => Some(Command::Nickname(params[0].to_val())),
            _ => { // ingame Move
                let mov = String::from_utf8_lossy(cmd).to_string();
                Some(Command::Move(mov))
            }
        }
    }
}

pub enum Response {
    Ok = 0x00,
    Err = 0x01,
}

struct Connection {
    client: TcpStream,
    nickname: String,
    chess: Option<Game>,
    read_buf: Buffer,
}

impl Connection {
    fn is_ingame(&self) -> bool {
        self.chess.is_some()
    }
    async fn wait_for_message(&mut self) -> bool {
        // read the first byte which indicates the length.
        // this value will be discarded and not be part of the read buffer
        let len = self.client.read_u8().await;
        let len = match len {
            Ok(n) => {
                if n as usize > BUF_LEN {
                    error!("message-length too big!: {}", n);
                    return false;
                }
                n
            }
            Err(e) => {
                error!("error at reading message length: {}", e);
                panic!("eof");
            }
        };
    
       self.read_buf.len = len as usize;
        // we got a new message so we clear our read buffer
        for i in 0..BUF_LEN {
            self.read_buf[i as usize] = 0;

        }
        // read the actual message into the read buffer
        let n = self
            .client
            .read_exact(&mut self.read_buf[..len as usize])
            .await;
        match n {
            Ok(0) => {
                info!("remote closed connection!");
                false
            } // connection closed
            Err(e) => {
                error!("Error at reading TcpStream: {}", e);
                false
            }
            Ok(n) => {
                trace!("Buffer Content: {} (Length: {n})", self.read_buf);
                true
            }
        }
    }

    fn new_game(&mut self, new_game: NewGame) {
        info!("New Game! (\"{}\" ({}) hoster side: {:?})", new_game.name, new_game.mode, new_game.hoster_side);
        self.chess = Some(Game::new());
    }

    fn exec(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Nickname(name) => {
                self.nickname = name;
                true
            }
            Command::NewGame(new_game) => {
                self.new_game(new_game);
                true
            }
            Command::JoinGame(id) => {
                self.join_game(id);
                true
            }
            Command::Move(mov) => {
                if self.is_ingame() {
                    let (src, dst) = if let Some(unpacked_mov) = mov.to_chess() {
                        unpacked_mov
                    } else {
                        warn!("cannot parse move: {style_bold}{fg_red}{}{style_reset}{fg_reset}", mov);
                        return false;
                    };
                    let chess = self.chess.as_mut().unwrap(); // we are ingame, so there must be a
                                                              // chess
                    if chess.make_move(src, dst) {
                        debug!("move {style_bold}{fg_green}{}{}{style_reset}{fg_reset} executed!", src, dst);
                        println!("{}", chess);
                        true
                    } else {
                        info!("illegal chess move");
                        false
                    }
                } else {
                    false // ingame but not a chess move
                }
            }
            Command::Invalid => {
                warn!("Invalid Command received!: {:?}", cmd);
                false
            }
        }
    }

    fn join_game(&self, id: String) {
        info!("Join Game. id: {:?}", id)
    }
    async fn run(&mut self) {
        println!("Hello, {}!", self.nickname);
        loop {
            // only reading the message, no further validation.
            // this blocks the task until a full message is available
            if self.wait_for_message().await {
            } else {
                error!("error while waiting for message!");
                continue;
            };

            // now we interpret the message
            let cmd = if let Some(cmd) = self.read_buf.parse() {
                cmd
            } else {
                error!("invalid command received!");
                continue;
            };

            if !self.exec(cmd) {
                info!("exec failed!");
                let _ = self.client.write(&[b'0']).await;
            } else {
                info!("exec succeed!");
                let _ = self.client.write(&[b'1']).await;
            }
        }
    }
}

pub struct Interface {
    _listener: Option<TcpListener>,
}

impl Interface {
    pub fn new() -> Interface {
        Interface { _listener: None }
    }

    pub async fn listen(&self) -> Result<()> {
        info!("Listening...");
        let listener = TcpListener::bind("127.0.0.1:7878").await?;

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("got connection from {}!", addr);
            let mut hndl = Connection {
                client: socket,
                nickname: String::new(),
                chess: None,
                read_buf: Buffer::new(),
            };
            tokio::spawn(async move {
                hndl.run().await;
            });
        }
    }
}
