use crate::game::Chess;
use crate::net::game::OnlineGame;
use crate::net::*;
use smol::channel::Receiver;
use std::collections::HashMap;

use super::server::ClientHandler;

pub struct GameManager {
    games: HashMap<GameId, OnlineGame>,
    pub clients: HashMap<ClientId, ClientHandler>, // Maps ClientId to their outbound message channel
    rx: Receiver<ClientToServerMessage>,           // receives messages from clients
    next_game_id: GameId,
}

impl GameManager {
    pub fn new(recv: Receiver<ClientToServerMessage>) -> Self {
        GameManager {
            games: HashMap::new(),
            clients: HashMap::new(),
            rx: recv,
            next_game_id: 0,
        }
    }

    fn create_game(&mut self, game_params: NewGameParams) -> OnlineGame {
        self.next_game_id += 1;

        log::info!(
            "create game with id: {} (mode: {})",
            self.next_game_id,
            game_params.mode
        );
        OnlineGame {
            id: self.next_game_id,
            chess: Chess::new(),
            _started: false,
            white_player: None,
            black_player: None,
            spectators: vec![],
            _time: game_params.time,
            _time_inc: game_params.time_inc,
        }
    }

    /*
     * Listen for messages from clients on the internal channel.
     *
     * This loop is responsible for managing chess games and players. Incoming messages will create
     * games, let player join games and let player make chess moves.
     * The results of these commands will then sent back to all clients that should be notified of
     * an executed command. E.g., when joining a game, the respective client will get an answer
     * wether joining was successful, but making a move, all players in the associated game will
     * get a message of the board state change.
     */
    pub async fn run(&mut self) {
        log::info!("Task started.");
        loop {
            match self.rx.recv().await {
                Ok(msg) => {
                    let client_id = msg.client_id;
                    log::info!("Received Message: {:?}", msg);
                    match msg.cmd {
                        Command::NewGame(game_params) => {
                            let game = self.create_game(game_params);
                            let id = game.id;
                            log::info!("created game with ID: {}", id);
                            self.games.insert(id, game);
                            // inform clients of new game
                            for c in &self.clients {
                                let _ =
                                    c.1.tx
                                        .send(ServerToClientMessage {
                                            msg: Response::GameCreated(id, client_id),
                                        })
                                        .await;
                            }
                        }
                        Command::JoinGame(join_params) => {
                            let game_id = join_params.game_id;
                            let game = match self.games.get_mut(&game_id) {
                                Some(g) => g,
                                None => {
                                    log::warn!(
                                        "got request to join game with invalid game id {game_id}"
                                    );
                                    return;
                                }
                            };
                            let side = join_params.side;
                            // TODO: return value should be response to client
                            let res = game.add_player(client_id, side);
                            match res {
                                Ok(side) => {
                                    let response = ServerToClientMessage {
                                        msg: Response::GameJoined(game_id, client_id, side),
                                    };
                                    let c = self.clients.get(&client_id);
                                    match c {
                                        Some(c) => _ = c.tx.send(response).await,
                                        None => {
                                            log::warn!(
                                                "sending to a client that is not registered!"
                                            )
                                        }
                                    }
                                }
                                Err(e) => log::warn!("error at adding player: {e}"),
                            }
                        }

                        Command::Move(game_id, mov) => {
                            let game = self.games.get_mut(&game_id);
                            match game {
                                Some(game) => {
                                    let changes = game.chess.make_move(mov);
                                    let clients = game.get_participants();
                                    for c in clients {
                                        let msg = ServerToClientMessage {
                                            msg: Response::Update(changes.clone()),
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
                        Command::Register(tx) => {
                            let client = ClientHandler::new(tx);
                            log::info!("client registered with ID: {}", client.id);
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
