use crate::chess::ChessColor;
use crate::{ChessError, NetError, NetResult};
use crate::{ChessMove, Tile, WoodPiece as Piece};
use crate::{ClientId, GameId};
use smol::channel::Sender;
use std::fmt;

/// A trivial Reader for network messages. Gives us the possibility to read
/// fields from the byte stream in a simpler way.
pub struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn read_u8(&mut self) -> NetResult<u8> {
        if self.offset < self.bytes.len() {
            let val = self.bytes[self.offset];
            self.offset += 1;
            Ok(val)
        } else {
            Err(NetError::Protocol("Unexpected end of data".to_string()))
        }
    }

    pub fn read_u32_le(&mut self) -> NetResult<u32> {
        if self.offset + 4 <= self.bytes.len() {
            let val =
                u32::from_le_bytes(self.bytes[self.offset..self.offset + 4].try_into().unwrap());
            self.offset += 4;
            Ok(val)
        } else {
            Err(NetError::Protocol("Unexpected end of data".to_string()))
        }
    }

    pub fn read_str(&mut self, len: usize) -> NetResult<&'a str> {
        if self.offset + len <= self.bytes.len() {
            let s = std::str::from_utf8(&self.bytes[self.offset..self.offset + len])
                .map_err(|_| NetError::Protocol("Failed to parse string".to_string()))?;
            self.offset += len;
            Ok(s)
        } else {
            Err(NetError::Protocol("Unexpected end of data".to_string()))
        }
    }

    pub fn remaining(&self) -> &[u8] {
        &self.bytes[self.offset..]
    }
}

#[derive(Debug, Clone)]
pub struct NewGameParams {
    pub mode: u8,
    pub time: u32,
    pub time_inc: u32,
}

impl NewGameParams {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.mode, 0];
        bytes.extend_from_slice(&self.time.to_le_bytes());
        bytes.extend_from_slice(&self.time_inc.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct JoinGameParams {
    pub game_id: u32,
    pub side: UserRoleSelection,
}

impl JoinGameParams {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.game_id.to_le_bytes());
        bytes.push(self.side as u8);
        bytes
    }
}

pub trait NetMessage: Sized {
    fn from_bytes(bytes: &[u8]) -> NetResult<Self>;
    fn to_bytes(&self) -> Vec<u8>;
}

/// Messages from a client to the server.
#[derive(Debug, Clone)]
pub enum ClientMessage {
    Register(Sender<ServerMessage>), // TODO: this is not an actual client-message
    SetNickname(String),
    NewGame(NewGameParams),
    JoinGame(JoinGameParams),
    Move(GameId, ChessMove),
    QueryGames,
    QueryGameDetails(GameId),
    QueryClientDetails(ClientId),
    LeaveGame(GameId),
}

impl ClientMessage {
    pub const NEW_GAME: u8 = 0x0A;
    pub const MAKE_MOVE: u8 = 0x0B;
    pub const QUERY_GAMES: u8 = 0x0C;
    pub const QUERY_GAME_DETAILS: u8 = 0x0D;
    pub const JOIN_GAME: u8 = 0x0E;
    pub const LEAVE_GAME: u8 = 0x0F;
    pub const SET_NICKNAME: u8 = 0x10;
    pub const QUERY_CLIENT_DETAILS: u8 = 0x11;
}

/// Serialization and deserialization for `ClientMessage`.
impl NetMessage for ClientMessage {
    fn from_bytes(bytes: &[u8]) -> NetResult<Self> {
        let mut reader = Reader::new(bytes);
        let opcode = reader.read_u8()?;

        match opcode {
            Self::NEW_GAME => {
                let mode = reader.read_u8()?;
                let time = reader.read_u32_le()?;
                let time_inc = reader.read_u32_le()?;

                let game_params = NewGameParams {
                    mode,
                    time,
                    time_inc,
                };
                Ok(ClientMessage::NewGame(game_params))
            }

            Self::JOIN_GAME => {
                let game_id = reader.read_u32_le()?;
                let side = UserRoleSelection::from_u8(reader.read_u8()?);

                let join_params = JoinGameParams { game_id, side };
                Ok(ClientMessage::JoinGame(join_params))
            }
            Self::QUERY_GAMES => Ok(ClientMessage::QueryGames),
            Self::MAKE_MOVE => {
                let game_id = reader.read_u32_le()?;

                let mov_str = String::from_utf8(reader.remaining().to_vec())
                    .map_err(|_| NetError::Protocol("Failed to parse move string".to_string()))?;

                let mov: ChessMove = mov_str.parse().map_err(|e: String| {
                    NetError::Protocol(format!("could not parse chess move!: {:?}", e))
                })?;
                Ok(ClientMessage::Move(game_id, mov))
            }
            Self::QUERY_GAME_DETAILS => {
                let game_id = reader.read_u32_le()?;
                Ok(ClientMessage::QueryGameDetails(game_id))
            }
            Self::QUERY_CLIENT_DETAILS => {
                let client_id = reader.read_u32_le()? as usize;
                Ok(ClientMessage::QueryClientDetails(client_id))
            }
            Self::LEAVE_GAME => {
                let gid = reader.read_u32_le()?;
                Ok(ClientMessage::LeaveGame(gid))
            }
            Self::SET_NICKNAME => {
                let nickname = String::from_utf8(reader.remaining().to_vec())
                    .map_err(|_| NetError::Protocol("Failed to parse nickname".to_string()))?;
                Ok(ClientMessage::SetNickname(nickname))
            }
            _ => Err(NetError::Protocol(format!(
                "parse: invalid command 0x{:02X}",
                opcode
            ))),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ClientMessage::NewGame(params) => {
                let mut data = vec![Self::NEW_GAME];
                data.extend_from_slice(&params.to_bytes());
                data
            }
            ClientMessage::JoinGame(params) => {
                let mut data = vec![Self::JOIN_GAME];
                data.extend_from_slice(&params.to_bytes());
                data
            }
            ClientMessage::QueryGames => vec![Self::QUERY_GAMES],
            ClientMessage::QueryGameDetails(game_id) => {
                let mut data = vec![Self::QUERY_GAME_DETAILS];
                data.extend_from_slice(&game_id.to_le_bytes());
                data
            }
            ClientMessage::QueryClientDetails(client_id) => {
                let mut data = vec![Self::QUERY_CLIENT_DETAILS];
                data.extend_from_slice(&(*client_id as u32).to_le_bytes());
                data
            }
            ClientMessage::Move(game_id, mov) => {
                let mut data = vec![Self::MAKE_MOVE];
                data.extend_from_slice(&game_id.to_le_bytes());
                data.extend_from_slice(mov.to_string().as_bytes());
                data
            }
            ClientMessage::Register(_) => {
                vec![]
            }
            ClientMessage::LeaveGame(gid) => {
                let mut data = vec![Self::LEAVE_GAME];
                data.extend_from_slice(&gid.to_le_bytes());
                data
            }
            ClientMessage::SetNickname(name) => {
                let mut data = vec![Self::SET_NICKNAME];
                data.extend_from_slice(name.to_string().as_bytes());
                data
            }
        }
    }
}

impl fmt::Display for ClientMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ClientMessage::NewGame(_) => "New Game",
            ClientMessage::JoinGame(_) => "Join Game",
            ClientMessage::Move(_, _) => "Make Chess Move",
            ClientMessage::QueryGames => "Query Games",
            ClientMessage::Register(_) => "Register Client",
            ClientMessage::QueryGameDetails(_) => "Query Game Details",
            ClientMessage::QueryClientDetails(_) => "Query Client Details",
            ClientMessage::LeaveGame(_) => "Leave Game",
            ClientMessage::SetNickname(_) => "Set Nickname",
        };
        write!(f, "{}", s)
    }
}

/// Messages from the server to the client.
#[derive(Debug, Clone)]
pub enum ServerMessage {
    Update(Vec<(Tile, Option<Piece>)>),
    GameCreated(GameId, ClientId),
    GameJoined(GameId, ClientId, UserRoleSelection, String),
    GameLeft(GameId, ClientId),
    IllegalMove(ChessError),
    GamesList(Vec<GameId>),
    GameDetails(GameId, Option<ClientId>, Option<ClientId>, u32, u32),
    ClientDetails(ClientId, String),
    Checkmate(GameId, ChessColor),
    Stalemate(GameId),
    LoginAccepted(ClientId),
}

impl ServerMessage {
    pub const GAME_CREATED: u8 = 0x81;
    pub const JOIN_GAME: u8 = 0x82;
    pub const GAME_LEFT: u8 = 0x84;
    pub const BOARD_UPDATED: u8 = 0x83;
    pub const ILLEGAL_MOVE: u8 = 0x85;
    pub const GAMES_LIST: u8 = 0x86;
    pub const CHECKMATE: u8 = 0x87;
    pub const STALEMATE: u8 = 0x88;
    pub const GAME_DETAILS: u8 = 0x8D;
    pub const CLIENT_DETAILS: u8 = 0x8E;
    pub const LOGIN_ACCEPTED: u8 = 0xF0;

    pub fn opcode(&self) -> u8 {
        match self {
            ServerMessage::GameCreated(_, _) => Self::GAME_CREATED,
            ServerMessage::GameJoined(_, _, _, _) => Self::JOIN_GAME,
            ServerMessage::Update(_) => Self::BOARD_UPDATED,
            ServerMessage::IllegalMove(_) => Self::ILLEGAL_MOVE,
            ServerMessage::GamesList(_) => Self::GAMES_LIST,
            ServerMessage::Checkmate(_, _) => Self::CHECKMATE,
            ServerMessage::Stalemate(_) => Self::STALEMATE,
            ServerMessage::GameDetails(_, _, _, _, _) => Self::GAME_DETAILS,
            ServerMessage::ClientDetails(_, _) => Self::CLIENT_DETAILS,
            ServerMessage::LoginAccepted(_) => Self::LOGIN_ACCEPTED,
            ServerMessage::GameLeft(_, _) => Self::GAME_LEFT,
        }
    }
}

/// Serialization and deserialization for `ServerMessage`.
impl NetMessage for ServerMessage {
    fn from_bytes(bytes: &[u8]) -> NetResult<Self> {
        let mut reader = Reader::new(bytes);
        let opcode_byte = reader.read_u8()?;

        match opcode_byte {
            Self::BOARD_UPDATED => {
                let mut updates = Vec::new();
                while reader.remaining().len() >= 3 {
                    let tile_str = reader.read_str(2)?;
                    let tile = Tile::from(tile_str);
                    let piece_char = reader.read_u8()? as char;
                    let piece = Piece::from_char(piece_char);
                    updates.push((tile, piece));
                }
                Ok(ServerMessage::Update(updates))
            }
            Self::GAME_CREATED => {
                let game_id = reader.read_u32_le()?;
                let client_id = reader.read_u32_le()? as usize;
                Ok(ServerMessage::GameCreated(game_id, client_id))
            }
            Self::JOIN_GAME => {
                let game_id = reader.read_u32_le()?;
                let client_id = reader.read_u32_le()? as usize;
                let side = UserRoleSelection::from_u8(reader.read_u8()?);
                let fen = String::from_utf8_lossy(reader.remaining()).to_string();
                Ok(ServerMessage::GameJoined(game_id, client_id, side, fen))
            }
            Self::ILLEGAL_MOVE => {
                let err = ChessError::from_bytes(reader.remaining()).map_err(|e| {
                    NetError::Protocol(format!("Failed to parse ChessError: {}", e))
                })?;
                Ok(ServerMessage::IllegalMove(err))
            }
            Self::GAMES_LIST => {
                let mut game_ids = Vec::new();
                while reader.remaining().len() >= 4 {
                    let id = reader.read_u32_le()?;
                    game_ids.push(id);
                }
                Ok(ServerMessage::GamesList(game_ids))
            }
            Self::CHECKMATE => {
                let game_id = reader.read_u32_le()?;
                let is_checkmated = if reader.read_u8()? == 0 {
                    ChessColor::Black
                } else {
                    ChessColor::White
                };
                Ok(ServerMessage::Checkmate(game_id, is_checkmated))
            }
            Self::STALEMATE => {
                let game_id = reader.read_u32_le()?;
                Ok(ServerMessage::Stalemate(game_id))
            }
            Self::GAME_DETAILS => {
                let game_id = reader.read_u32_le()?;
                let white_id = reader.read_u32_le()?;
                let white_id_opt = if white_id > 0 {
                    Some(white_id as usize)
                } else {
                    None
                };
                let black_id = reader.read_u32_le()?;
                let black_id_opt = if black_id > 0 {
                    Some(black_id as usize)
                } else {
                    None
                };
                let time = reader.read_u32_le()?;
                let inc = reader.read_u32_le()?;

                Ok(ServerMessage::GameDetails(
                    game_id,
                    white_id_opt,
                    black_id_opt,
                    time,
                    inc,
                ))
            }
            Self::CLIENT_DETAILS => {
                let client_id = reader.read_u32_le()? as usize;
                let name = String::from_utf8(reader.remaining().to_vec())
                    .map_err(|_| NetError::Protocol("Failed to parse nickname".to_string()))?;
                Ok(ServerMessage::ClientDetails(client_id, name))
            }
            Self::GAME_LEFT => {
                let game_id = reader.read_u32_le()?;
                let client_id = reader.read_u32_le()? as usize;
                Ok(ServerMessage::GameLeft(game_id, client_id))
            }
            Self::LOGIN_ACCEPTED => {
                let client_id = reader.read_u32_le()? as usize;
                Ok(ServerMessage::LoginAccepted(client_id))
            }
            _ => Err(NetError::Protocol(format!(
                "Unknown opcode: {}",
                opcode_byte
            ))),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ServerMessage::Update(tiles) => {
                let mut msg = vec![Self::BOARD_UPDATED];
                for u in tiles {
                    let mut tile = u.0.to_string().as_bytes().to_owned();
                    let piece = match u.1 {
                        Some(p) => p.as_byte(),
                        None => 'X',
                    };
                    msg.append(&mut tile);
                    msg.push(piece as u8);
                }
                msg
            }
            ServerMessage::IllegalMove(err) => {
                let mut data = vec![Self::ILLEGAL_MOVE];
                data.extend_from_slice(&err.to_bytes());
                data
            }
            ServerMessage::GameCreated(game_id, client_id) => {
                let mut data = vec![Self::GAME_CREATED];
                data.extend_from_slice(&game_id.to_le_bytes());
                data.extend_from_slice(&(*client_id as u32).to_le_bytes());
                data
            }
            ServerMessage::GameJoined(game_id, client_id, side, fen) => {
                let mut data = vec![Self::JOIN_GAME];
                data.extend_from_slice(&game_id.to_le_bytes());
                data.extend_from_slice(&(*client_id as u32).to_le_bytes());
                data.push(*side as u8);
                data.extend_from_slice(fen.as_bytes());
                data
            }
            ServerMessage::GamesList(game_ids) => {
                let mut data = vec![Self::GAMES_LIST];
                for id in game_ids {
                    data.extend_from_slice(&id.to_le_bytes());
                }
                data
            }
            ServerMessage::Checkmate(game_id, is_checkmated) => {
                let mut data = vec![Self::CHECKMATE];
                data.extend_from_slice(&game_id.to_le_bytes());
                data.push(match is_checkmated {
                    ChessColor::Black => 0,
                    ChessColor::White => 1,
                });
                data
            }
            ServerMessage::Stalemate(game_id) => {
                let mut data = vec![Self::STALEMATE];
                data.extend_from_slice(&game_id.to_le_bytes());
                data
            }
            ServerMessage::LoginAccepted(client_id) => {
                let mut data = vec![Self::LOGIN_ACCEPTED];
                data.extend_from_slice(&(*client_id as u32).to_le_bytes());
                data
            }
            ServerMessage::GameDetails(game_id, white_id, black_id, time, inc) => {
                let mut data = vec![Self::GAME_DETAILS];
                data.extend_from_slice(&game_id.to_le_bytes());
                let white_id = white_id.map(|id| id as u32).unwrap_or(0);
                data.extend_from_slice(&white_id.to_le_bytes());
                let black_id = black_id.map(|id| id as u32).unwrap_or(0);
                data.extend_from_slice(&black_id.to_le_bytes());
                data.extend_from_slice(&time.to_le_bytes());
                data.extend_from_slice(&inc.to_le_bytes());
                data
            }
            ServerMessage::ClientDetails(client_id, name) => {
                let mut data = vec![Self::CLIENT_DETAILS];
                data.extend_from_slice(&(*client_id as u32).to_le_bytes());
                data.extend_from_slice(name.as_bytes());
                data
            }
            ServerMessage::GameLeft(game_id, client_id) => {
                let mut data = vec![Self::GAME_LEFT];
                data.extend_from_slice(&game_id.to_le_bytes());
                data.extend_from_slice(&client_id.to_le_bytes());
                data
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRoleSelection {
    Black = 0,
    White = 1,
    Random = 2,
    Spectator = 3,
    Both = 4,
}

impl UserRoleSelection {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => UserRoleSelection::Black,
            1 => UserRoleSelection::White,
            2 => UserRoleSelection::Random,
            3 => UserRoleSelection::Spectator,
            4 => UserRoleSelection::Both,
            _ => UserRoleSelection::Spectator,
        }
    }
}
