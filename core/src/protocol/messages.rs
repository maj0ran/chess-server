use crate::protocol::{JoinGameParams, NewGameParams, UserRoleSelection};
use crate::states::GameOverReason;
use crate::*;
use smol::channel::Sender;
use std::fmt;

///=======================================///
/// Messages from a client to the server. ///
///=======================================///

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
    QueryBoard(GameId),
    QueryMoveHistory(GameId),
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
    pub const QUERY_BOARD: u8 = 0x12;
    pub const QUERY_MOVE_HISTORY: u8 = 0x13;
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
            ClientMessage::QueryBoard(_) => "Query Board",
            ClientMessage::QueryMoveHistory(_) => "Query Move History",
        };
        write!(f, "{}", s)
    }
}

///=======================================///
/// Messages from the server to a client. ///
///=======================================///

#[derive(Debug, Clone)]
pub enum ServerMessage {
    MoveAccepted(u8, String, Vec<(Tile, Option<WoodPiece>)>), // len(SAN), SAN, [updates tiles]
    GameCreated(GameId, ClientId),
    GameJoined(GameId, ClientId, UserRoleSelection),
    GameLeft(GameId, ClientId),
    IllegalMove(ChessError),
    GamesList(Vec<GameId>),
    GameDetails(GameId, Option<ClientId>, Option<ClientId>, u32, u32),
    ClientDetails(ClientId, String),
    GameOver(GameId, GameOverReason),
    LoginAccepted(ClientId),
    BoardState(GameId, String),
    MoveHistory(GameId, Vec<String>),
}

impl ServerMessage {
    pub const GAME_CREATED: u8 = 0x81;
    pub const GAME_JOINED: u8 = 0x82;
    pub const GAME_LEFT: u8 = 0x84;
    pub const MOVE_ACCEPTED: u8 = 0x83;
    pub const ILLEGAL_MOVE: u8 = 0x85;
    pub const GAMES_LIST: u8 = 0x86;
    pub const GAME_OVER: u8 = 0x87;
    pub const GAME_DETAILS: u8 = 0x8D;
    pub const CLIENT_DETAILS: u8 = 0x8E;
    pub const BOARD_STATE: u8 = 0x8F;
    pub const MOVE_HISTORY: u8 = 0x90;
    pub const LOGIN_ACCEPTED: u8 = 0xF0;

    pub fn opcode(&self) -> u8 {
        match self {
            ServerMessage::GameCreated(_, _) => Self::GAME_CREATED,
            ServerMessage::GameJoined(_, _, _) => Self::GAME_JOINED,
            ServerMessage::MoveAccepted(_, _, _) => Self::MOVE_ACCEPTED,
            ServerMessage::IllegalMove(_) => Self::ILLEGAL_MOVE,
            ServerMessage::GamesList(_) => Self::GAMES_LIST,
            ServerMessage::GameOver(_, _) => Self::GAME_OVER,
            ServerMessage::GameDetails(_, _, _, _, _) => Self::GAME_DETAILS,
            ServerMessage::ClientDetails(_, _) => Self::CLIENT_DETAILS,
            ServerMessage::LoginAccepted(_) => Self::LOGIN_ACCEPTED,
            ServerMessage::GameLeft(_, _) => Self::GAME_LEFT,
            ServerMessage::BoardState(_, _) => Self::BOARD_STATE,
            ServerMessage::MoveHistory(_, _) => Self::MOVE_HISTORY,
        }
    }
}
