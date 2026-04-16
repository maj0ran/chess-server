use crate::chess::ChessColor;
use crate::protocol::messages::{ClientMessage, ServerMessage};
use crate::protocol::{JoinGameParams, NewGameParams, Reader, UserRoleSelection};
use crate::states::GameOverReason;
use crate::{ChessError, NetError, NetResult};
use crate::{ChessMove, Tile, WoodPiece as Piece};

pub trait NetMessage: Sized {
    fn from_bytes(bytes: &[u8]) -> NetResult<Self>;
    fn to_bytes(&self) -> Vec<u8>;
}

///========================================================///
/// Serialization and deserialization for `ClientMessage`. ///
///========================================================///

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
                let gid = reader.read_u32_le()?;
                let side = UserRoleSelection::from_u8(reader.read_u8()?);

                let join_params = JoinGameParams { game_id: gid, side };
                Ok(ClientMessage::JoinGame(join_params))
            }
            Self::QUERY_GAMES => Ok(ClientMessage::QueryGames),
            Self::MAKE_MOVE => {
                let gid = reader.read_u32_le()?;

                let mov_str = String::from_utf8(reader.remaining().to_vec())
                    .map_err(|_| NetError::Protocol("Failed to parse move string".to_string()))?;

                let mov: ChessMove = mov_str.parse().map_err(|e: String| {
                    NetError::Protocol(format!("could not parse chess move!: {:?}", e))
                })?;
                Ok(ClientMessage::Move(gid, mov))
            }
            Self::QUERY_GAME_DETAILS => {
                let gid = reader.read_u32_le()?;
                Ok(ClientMessage::QueryGameDetails(gid))
            }
            Self::QUERY_CLIENT_DETAILS => {
                let cid = reader.read_u32_le()? as usize;
                Ok(ClientMessage::QueryClientDetails(cid))
            }
            Self::LEAVE_GAME => Ok(ClientMessage::LeaveGame),
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
            ClientMessage::QueryGameDetails(gid) => {
                let mut data = vec![Self::QUERY_GAME_DETAILS];
                data.extend_from_slice(&gid.to_le_bytes());
                data
            }
            ClientMessage::QueryClientDetails(cid) => {
                let mut data = vec![Self::QUERY_CLIENT_DETAILS];
                data.extend_from_slice(&(*cid as u32).to_le_bytes());
                data
            }
            ClientMessage::Move(gid, mov) => {
                let mut data = vec![Self::MAKE_MOVE];
                data.extend_from_slice(&gid.to_le_bytes());
                data.extend_from_slice(mov.to_string().as_bytes());
                data
            }
            ClientMessage::Register(_) => {
                vec![]
            }
            ClientMessage::LeaveGame => vec![Self::LEAVE_GAME],
            ClientMessage::SetNickname(name) => {
                let mut data = vec![Self::SET_NICKNAME];
                data.extend_from_slice(name.to_string().as_bytes());
                data
            }
        }
    }
}

///========================================================///
/// Serialization and deserialization for `ServerMessage`. ///
///========================================================///

impl NetMessage for ServerMessage {
    fn from_bytes(bytes: &[u8]) -> NetResult<Self> {
        let mut reader = Reader::new(bytes);
        let opcode_byte = reader.read_u8()?;

        match opcode_byte {
            Self::MOVE_ACCEPTED => {
                let san_len = reader.read_u8()?;
                let san = reader.read_str(san_len as usize)?.to_string();
                let mut updates = Vec::new();
                while reader.remaining().len() >= 3 {
                    let tile_str = reader.read_str(2)?;
                    let tile = Tile::from(tile_str);
                    let piece_char = reader.read_u8()? as char;
                    let piece = Piece::from_char(piece_char);
                    updates.push((tile, piece));
                }
                Ok(ServerMessage::MoveAccepted(san_len, san, updates))
            }
            Self::GAME_CREATED => {
                let gid = reader.read_u32_le()?;
                let cid = reader.read_u32_le()? as usize;
                Ok(ServerMessage::GameCreated(gid, cid))
            }
            Self::JOIN_GAME => {
                let gid = reader.read_u32_le()?;
                let cid = reader.read_u32_le()? as usize;
                let side = UserRoleSelection::from_u8(reader.read_u8()?);
                let fen = String::from_utf8_lossy(reader.remaining()).to_string();
                Ok(ServerMessage::GameJoined(gid, cid, side, fen))
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
                    let gid = reader.read_u32_le()?;
                    game_ids.push(gid);
                }
                Ok(ServerMessage::GamesList(game_ids))
            }
            Self::GAME_OVER => {
                let gid = reader.read_u32_le()?;
                let reason_byte = reader.read_u8()?;
                let winner_byte = reader.read_u8()?;
                let winner = match winner_byte {
                    0 => ChessColor::Black,
                    1 => ChessColor::White,
                    _ => ChessColor::White, // fallback
                };

                let reason = match reason_byte {
                    1 => GameOverReason::Checkmate(winner),
                    2 => GameOverReason::Resignation(winner),
                    3 => GameOverReason::TimeOut(winner),
                    4 => GameOverReason::Stalemate,
                    5 => GameOverReason::ThreefoldRepetition,
                    6 => GameOverReason::InsufficientMaterial,
                    7 => GameOverReason::FiftyMovesRule,
                    _ => panic!("Invalid game over reason"),
                };
                Ok(ServerMessage::GameOver(gid, reason))
            }
            Self::GAME_DETAILS => {
                let gid = reader.read_u32_le()?;
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
                    gid,
                    white_id_opt,
                    black_id_opt,
                    time,
                    inc,
                ))
            }
            Self::CLIENT_DETAILS => {
                let cid = reader.read_u32_le()? as usize;
                let name = String::from_utf8(reader.remaining().to_vec())
                    .map_err(|_| NetError::Protocol("Failed to parse nickname".to_string()))?;
                Ok(ServerMessage::ClientDetails(cid, name))
            }
            Self::GAME_LEFT => {
                let gid = reader.read_u32_le()?;
                let cid = reader.read_u32_le()? as usize;
                Ok(ServerMessage::GameLeft(gid, cid))
            }
            Self::LOGIN_ACCEPTED => {
                let cid = reader.read_u32_le()? as usize;
                Ok(ServerMessage::LoginAccepted(cid))
            }
            _ => Err(NetError::Protocol(format!(
                "Unknown opcode: {}",
                opcode_byte
            ))),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            ServerMessage::MoveAccepted(san_len, san, tiles) => {
                let mut msg = vec![Self::MOVE_ACCEPTED];
                msg.push(*san_len);
                msg.extend_from_slice(san.as_bytes());
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
            ServerMessage::GameCreated(gid, cid) => {
                let mut data = vec![Self::GAME_CREATED];
                data.extend_from_slice(&gid.to_le_bytes());
                data.extend_from_slice(&(*cid as u32).to_le_bytes());
                data
            }
            ServerMessage::GameJoined(gid, cid, side, fen) => {
                let mut data = vec![Self::JOIN_GAME];
                data.extend_from_slice(&gid.to_le_bytes());
                data.extend_from_slice(&(*cid as u32).to_le_bytes());
                data.push(*side as u8);
                data.extend_from_slice(fen.as_bytes());
                data
            }
            ServerMessage::GamesList(game_ids) => {
                let mut data = vec![Self::GAMES_LIST];
                for gid in game_ids {
                    data.extend_from_slice(&gid.to_le_bytes());
                }
                data
            }
            ServerMessage::GameOver(gid, reason) => {
                let mut data = vec![Self::GAME_OVER];
                data.extend_from_slice(&gid.to_le_bytes());
                data.push(reason.to_u8());
                let winner_byte = match reason.get_winner() {
                    Some(ChessColor::Black) => 0,
                    Some(ChessColor::White) => 1,
                    None => 2,
                };
                data.push(winner_byte);
                data
            }
            ServerMessage::LoginAccepted(cid) => {
                let mut data = vec![Self::LOGIN_ACCEPTED];
                data.extend_from_slice(&(*cid as u32).to_le_bytes());
                data
            }
            ServerMessage::GameDetails(gid, white_id, black_id, time, inc) => {
                let mut data = vec![Self::GAME_DETAILS];
                data.extend_from_slice(&gid.to_le_bytes());
                let white_id = white_id.map(|id| id as u32).unwrap_or(0);
                data.extend_from_slice(&white_id.to_le_bytes());
                let black_id = black_id.map(|id| id as u32).unwrap_or(0);
                data.extend_from_slice(&black_id.to_le_bytes());
                data.extend_from_slice(&time.to_le_bytes());
                data.extend_from_slice(&inc.to_le_bytes());
                data
            }
            ServerMessage::ClientDetails(cid, name) => {
                let mut data = vec![Self::CLIENT_DETAILS];
                data.extend_from_slice(&(*cid as u32).to_le_bytes());
                data.extend_from_slice(name.as_bytes());
                data
            }
            ServerMessage::GameLeft(gid, cid) => {
                let mut data = vec![Self::GAME_LEFT];
                data.extend_from_slice(&gid.to_le_bytes());
                data.extend_from_slice(&cid.to_le_bytes());
                data
            }
        }
    }
}
