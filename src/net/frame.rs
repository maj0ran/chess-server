use std::io::Read;

use crate::net::Command;
use crate::net::NewGame;
use crate::net::PlayerSideRequest;
use crate::net::*;

use super::Parameter;
use super::BUF_LEN;

pub struct Frame {
    pub len: u8,
    pub content: [u8; BUF_LEN],
}

/*
* Frame Format:
* first Byte is either
* - 0xA 0x32 <param> to create a new game with params
* params are seperated by 0x32 (space)
* - [a-h] followed by another 3-4 bytes to indicate a chess move like d2d4
*/
impl Frame {
    //  pub fn create(content: &[u8]) -> Frame {
    //      let len = content.len() as u8;
    //  }

    pub fn parse(&self) -> Option<Command> {
        let tokens: Vec<&[u8]> = self.content[..self.len as usize]
            .split(|s| &b' ' == s)
            .collect();
        let cmd = tokens[0];
        let params = &tokens[1..];
        if cmd.len() == 0 {
            log::warn!("got message but no content");
            return None;
        }

        let ret = match cmd[0] {
            NEW_GAME => {
                if params.len() != 2 {
                    log::error!("host: invalid number of params received!: {}", params.len());
                    return None;
                }
                let mode = params[0].to_val();
                let side: u8 = params[1].to_val();
                let side = PlayerSideRequest::try_from(side);
                let side = match side {
                    Ok(s) => s,
                    Err(_) => {
                        log::warn!("invalid side chosen! default to random");
                        PlayerSideRequest::Random
                    }
                };
                let new_game = NewGame::new(mode, side);
                Some(Command::NewGame(new_game))
            }

            JOIN_GAME => {
                if params.len() != 1 {
                    log::error!("join: invalid number of params received!: {}", params.len());
                    return None;
                }
                let game_name = params[0].to_val();
                Some(Command::JoinGame(game_name))
            }
            SET_NAME => Some(Command::Nickname(params[0].to_val())),
            _ => {
                // ingame Move
                let mov = String::from_utf8_lossy(cmd).to_string();
                Some(Command::Move(mov))
            }
        };
        // we got a new message so we clear our read buffer
        // //for i in 0..BUF_LEN {
        //     self[i as usize] = 0;
        // };
        ret
    }
}
