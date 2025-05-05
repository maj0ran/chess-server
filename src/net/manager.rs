use crate::net::game::OnlineGame;
use crate::net::*;
use smol::channel::Receiver;
use std::collections::HashMap;

use super::server::Client;

pub struct GameManager {
    games: HashMap<GameId, OnlineGame>,
    pub clients: HashMap<ClientId, Client>, // Maps ClientId to their outbound message channel
    recv: Receiver<ServerMessage>,          // receives messages from clients
    next_game_id: GameId,
    next_client_id: ClientId,
}

impl GameManager {
    pub fn new(recv: Receiver<ServerMessage>) -> Self {
        GameManager {
            games: HashMap::new(),
            clients: HashMap::new(),
            recv,
            next_game_id: 0,
            next_client_id: 0, // Start client IDs from 1
        }
    }

    fn create_game(&mut self) -> OnlineGame {
        self.next_game_id += 1;
        OnlineGame::new(self.next_game_id)
    }

    pub async fn run(&mut self) {
        log::info!("Task started.");
        loop {
            match self.recv.recv().await {
                Ok(msg) => {
                    let client_id = msg.client_id;
                    log::info!("Received Message: {:?}", msg);
                    match msg.cmd {
                        Command::NewGame(new_game) => {
                            let mut game = self.create_game();
                            match new_game.hoster_side {
                                PlayerSideRequest::Black => game.black_player = Some(client_id),
                                PlayerSideRequest::White => game.white_player = Some(client_id),
                                PlayerSideRequest::Random => todo!(),
                            }
                            log::info!("[GameManager] created game with ID: {}", game.id);
                            self.games.insert(game.id, game);
                        }
                        Command::JoinGame(_) => todo!(),
                        Command::Nickname(_) => todo!(),
                        Command::Move(game_id, mov) => {
                            let game = self.games.get_mut(&game_id);
                            match game {
                                Some(game) => {
                                    let changes = game.chess.make_move(mov);
                                    let clients = game.get_participants();
                                    for c in clients {
                                        let msg = ServerMessage {
                                            client_id: c,
                                            cmd: Command::Update(changes.clone()),
                                        };
                                        match self.clients.get(&c).unwrap().tx.send(msg).await {
                                            Ok(_) => {
                                                log::debug!("sending board update to: {}", c);
                                            }
                                            Err(e) => log::error!(
                                                "error sending update messsage to client #{}: {}",
                                                c,
                                                e
                                            ),
                                        }
                                    }
                                }
                                None => {
                                    log::warn!("got move command for invalid game ID!")
                                }
                            }
                        }
                        Command::Update(_) => log::warn!(
                            "got update message from client but only server should send this."
                        ),
                        Command::Register(tx) => {
                            let client = Client::new(tx);
                            log::info!("[GameManager] client registered with ID: {}", client.id);
                            self.clients.insert(client_id, client);
                        }
                        Command::_Invalid => todo!(),
                    }
                }
                Err(_) => {
                    log::info!("Client command channel closed. Shutting down.");
                    break;
                }
            }
        }
        println!("Task stopped.");
    }
}
