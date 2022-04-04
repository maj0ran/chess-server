use crate::chess::ChessMove;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum NetError {
    Io(io::Error),
    Protocol(String),
    Disconnected,
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::Io(e) => write!(f, "IO error: {}", e),
            NetError::Protocol(s) => write!(f, "Protocol error: {}", s),
            NetError::Disconnected => write!(f, "Client disconnected"),
        }
    }
}

impl std::error::Error for NetError {}

impl From<io::Error> for NetError {
    fn from(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::UnexpectedEof
            || error.kind() == io::ErrorKind::ConnectionAborted
            || error.kind() == io::ErrorKind::ConnectionReset
        {
            NetError::Disconnected
        } else {
            NetError::Io(error)
        }
    }
}

pub type NetResult<T> = std::result::Result<T, NetError>;

#[derive(Debug, Clone)]
pub enum ChessError {
    IllegalMove(ChessMove),
    NotYourTurn,
}

impl fmt::Display for ChessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChessError::IllegalMove(m) => write!(
                f,
                "Illegal move: {}{}{}{}{}",
                m.src.file,
                m.src.rank,
                m.dst.file,
                m.dst.rank,
                if let Some(s) = m.special {
                    s.to_string()
                } else {
                    "".to_string()
                }
            ),
            ChessError::NotYourTurn => write!(f, "It's not your turn"),
        }
    }
}

impl std::error::Error for ChessError {}

impl ChessError {
    /// Serialize ChessError to bytes for network transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ChessError::IllegalMove(mov) => {
                let mut bytes = vec![0u8]; // discriminant for IllegalMove
                bytes.extend_from_slice(mov.to_string().as_bytes());
                bytes
            }
            ChessError::NotYourTurn => {
                vec![1u8] // discriminant for NotYourTurn
            }
        }
    }

    /// Deserialize ChessError from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Empty ChessError bytes".to_string());
        }

        match bytes[0] {
            0 => {
                // IllegalMove
                let mov_str = String::from_utf8(bytes[1..].to_vec())
                    .map_err(|_| "Failed to parse move string".to_string())?;
                let mov = mov_str
                    .parse()
                    .map_err(|_| format!("Failed to parse ChessMove: {}", mov_str))?;
                Ok(ChessError::IllegalMove(mov))
            }
            1 => Ok(ChessError::NotYourTurn),
            _ => Err(format!("Unknown ChessError discriminant: {}", bytes[0])),
        }
    }
}

pub type ChessResult<T> = std::result::Result<T, ChessError>;

#[derive(Debug)]
pub enum GameManagerError {
    GameNotFound(u32),
    InvalidGameStatus(String),
}

impl fmt::Display for GameManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameManagerError::GameNotFound(id) => write!(f, "Game not found: {}", id),
            GameManagerError::InvalidGameStatus(s) => write!(f, "Invalid game status: {}", s),
        }
    }
}

impl std::error::Error for GameManagerError {}

pub type GameManagerResult<T> = std::result::Result<T, GameManagerError>;
