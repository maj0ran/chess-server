#[allow(unused)]
use log::{debug, error, info, trace, warn};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::game::Game;
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

#[derive(Debug)]
#[repr(u8)]
enum Command {
    NewGame(NewGame),
    JoinGame(String),
    Nickname(String),
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
impl Message for [u8; BUF_LEN] {
    fn parse(&self) -> Option<Command> {
        let tokens: Vec<&[u8]> = self.split(|s| &b' ' == s).collect();
        println!("{:?}", tokens);

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
            _ => {
                panic!()
            }
        }
    }
}

pub enum Response {
    Ok = 0x01,
}

struct Connection {
    client: TcpStream,
    nickname: String,
    chess: Option<Game>,
}

impl Connection {
    async fn read_message(&mut self) -> Option<[u8; BUF_LEN]> {
        let mut buffer = [0u8; BUF_LEN]; // TODO: make this more global instead of returning
        let len = self.client.read_u8().await;

        let len = match len {
            Ok(n) => {
                if n as usize > BUF_LEN {
                    error!("message-length too big!: {}", n);
                    return None;
                }
                n
            }
            Err(e) => {
                error!("error at reading message length: {}", e);
                panic!("eof");
            }
        };

        let n = self.client.read_exact(&mut buffer[..len as usize]).await;
        match n {
            Ok(0) => {
                info!("remote closed connection!");
                None
            } // connection closed
            Err(e) => {
                error!("Error at reading TcpStream: {}", e);
                None
            }
            Ok(n) => {
                debug!("read {n} bytes");
                Some(buffer)
            }
        }
    }

    fn new_game(&self, new_game: NewGame) {
        info!("New Game!");
        info!("name: {:?}", new_game.name);
        info!("side: {:?}", new_game.hoster_side);
        info!("mode: {:?}", new_game.mode);
        let chess = Game::new();
    }

    fn join_game(&self, id: String) {
        info!("Join Game. id: {:?}", id)
    }
    async fn run(&mut self) {
        println!("Hello, {}!", self.nickname);
        loop {
            // only reading the message, no further validation.
            // this blocks the task until a full message is available
            let msg = if let Some(msg) = self.read_message().await {
                msg
            } else {
                warn!("could not read message!");
                continue;
            };

            // now we interpret the message
            let cmd = if let Some(cmd) = msg.parse() {
                cmd
            } else {
                error!("invalid command received!");
                continue;
            };

            match cmd {
                Command::Nickname(name) => self.nickname = name,
                Command::NewGame(new_game) => self.new_game(new_game),
                Command::JoinGame(id) => self.join_game(id),
                Command::Invalid => warn!("Invalid Command received!: {:?}", msg),
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
            };
            tokio::spawn(async move {
                hndl.run().await;
            });
        }
    }
}
