use std::io::ErrorKind;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::game::Chess;
use crate::net::*;

pub struct OnlineGame {
    pub id: GameId,
    pub chess: Chess,
    pub _started: bool,
    pub white_player: Option<ClientId>,
    pub black_player: Option<ClientId>,
    pub spectators: Vec<ClientId>,

    pub _time: u32,
    pub _time_inc: u32,
}

impl OnlineGame {
    pub fn _start_game(&mut self) -> bool {
        if self.white_player.is_some() && self.black_player.is_some() && !self._started {
            self._started = true;
            true
        } else {
            false
        }
    }

    pub fn add_player(&mut self, client_id: ClientId, side: PlayerSideRequest) -> Result<()> {
        match side {
            PlayerSideRequest::Black => {
                if let Some(_) = self.black_player {
                    Err(std::io::Error::new(ErrorKind::Other, "invalid side"))
                } else {
                    self.black_player = Some(client_id);
                    Ok(())
                }
            }
            PlayerSideRequest::White => {
                if let Some(_) = self.white_player {
                    Err(std::io::Error::new(ErrorKind::Other, "invalid side"))
                } else {
                    self.white_player = Some(client_id);
                    Ok(())
                }
            }
            PlayerSideRequest::Random => {
                if self.white_player != None && self.black_player != None {
                    return Err(std::io::Error::new(ErrorKind::Other, "game already full"));
                };

                if let Some(_) = self.white_player {
                    self.black_player = Some(client_id);
                }

                if let Some(_) = self.black_player {
                    self.white_player = Some(client_id);
                }

                let side: bool = (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos()
                    % 2)
                    != 0;

                match side {
                    false => {
                        self.black_player = Some(client_id);
                        Ok(())
                    }
                    true => {
                        self.white_player = Some(client_id);
                        Ok(())
                    }
                }
            }
            PlayerSideRequest::Spectator => {
                self.spectators.push(client_id);
                Ok(())
            }
            PlayerSideRequest::Both => todo!(),
        }
    }

    pub fn get_participants(&self) -> impl Iterator<Item = ClientId> + '_ {
        self.white_player
            .iter()
            .chain(self.black_player.iter())
            .chain(self.spectators.iter())
            .copied()
    }
}
