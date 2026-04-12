use crate::chess::chess::Chess;
use crate::chess::san::San;
use crate::server::chessgame::{ChessGame, ChessGameOutcome, ChessGameState};
use chess_core::*;
use chrono::prelude::*;
use smol::channel::{Receiver, Sender};
use smol::fs::File;
use smol::io::AsyncWriteExt;
use std::collections::HashMap;

/// The endpoint of a client for the `GameManager`.
/// Those are used by the `GameManager` to keep a connection
/// to all `ClientSession` via channels.
/// Since the clients all run in their own task, they cannot be
/// directly accessed by the `GameManager`. So we use channels
/// to let messages flow from the `GameManager` task to the
/// individual `ClientSession` tasks.
pub struct ClientEndpoint {
    pub tx: Sender<ServerMessage>,
    pub name: String,
    pub in_game: Option<GameId>,
}

impl ClientEndpoint {
    pub fn new(tx: Sender<ServerMessage>) -> Self {
        ClientEndpoint {
            tx,
            name: String::from(""),
            in_game: None,
        }
    }
}

/// The `GameManager` is responsible for managing all games and communicating
/// game states to the clients.
/// The manager has one receiver channel which is used by all `ClientSessions`
/// and N transmitter channels, one for each `ClientSession`.
///
/// The `GameManager` receives `ClientMessages` for creating games, assigning clients to games
/// (i.e., letting clients join games), querying games, and making moves. It will also send
/// back `ServerMessages` for whatever happened in games, i.e., if a game was created, joined
/// or a move was successfully executed (or rejected).
pub struct GameManager {
    games: HashMap<GameId, ChessGame>,
    pub clients: HashMap<ClientId, ClientEndpoint>, // Maps ClientId to their outbound message channel
    rx: Receiver<(ClientId, ClientMessage)>,        // receives messages from clients
    next_game_id: GameId,
}

impl GameManager {
    /// constructor
    pub fn new(recv: Receiver<(ClientId, ClientMessage)>) -> Self {
        GameManager {
            games: HashMap::new(),
            clients: HashMap::new(),
            rx: recv,
            next_game_id: 0,
        }
    }

    /// Creates a new `ChessGame` and returns it.
    /// The game is not yet assigned to any client.
    fn create_game(&mut self, game_params: NewGameParams) -> ChessGame {
        let id = self.next_game_id;
        self.next_game_id += 1;

        log::info!("create game with id: {} (mode: {})", id, game_params.mode);
        ChessGame {
            id,
            chess: Chess::new(),
            _started: false,
            white_player: None,
            black_player: None,
            spectators: vec![],
            _time: game_params.time,
            _time_inc: game_params.time_inc,
            move_history: vec![],
        }
    }

    /// The main loop of the `GameManager`.
    /// Here, the GM listens for `ClientMessages` from `ClientSessions` on its receiver channel.
    /// It will then process the message and send back `ServerMessages` to the `ClientSessions`.
    pub async fn run(&mut self) {
        loop {
            match self.rx.recv().await {
                Ok((client_id, cmd)) => {
                    // the actual command
                    match cmd {
                        ClientMessage::NewGame(game_params) => {
                            self.handle_new_game(client_id, game_params).await;
                        }
                        ClientMessage::JoinGame(join_params) => {
                            self.handle_join_game(client_id, join_params).await;
                        }
                        ClientMessage::Register(tx) => {
                            self.handle_register(client_id, tx).await;
                        }
                        ClientMessage::Move(game_id, mov) => {
                            self.handle_move(client_id, game_id, mov).await;
                        }
                        ClientMessage::QueryGames => {
                            self.handle_query_games(client_id).await;
                        }
                        ClientMessage::QueryGameDetails(game_id) => {
                            self.handle_query_game_details(client_id, game_id).await;
                        }
                        ClientMessage::QueryClientDetails(client_id_query) => {
                            self.handle_query_client_details(client_id, client_id_query)
                                .await;
                        }
                        ClientMessage::LeaveGame => {
                            self.handle_leave_game(client_id).await;
                        }
                        ClientMessage::SetNickname(name) => {
                            log::info!("Set nickname for client {} to {}", client_id, name);
                            self.handle_set_nickname(client_id, name).await;
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    }
    /// Create a new game and inform all `ClientSessions` about it.
    async fn handle_new_game(&mut self, client_id: ClientId, game_params: NewGameParams) {
        let game = self.create_game(game_params);
        let id = game.id;
        self.games.insert(id, game);
        // inform all clients of new game
        for c in &self.clients {
            let _ = c.1.tx.send(ServerMessage::GameCreated(id, client_id)).await;
        }
    }

    /// Assign a client to a game and inform the `ClientSession` about it.
    /// TODO: should inform all clients that are connected to the game.
    async fn handle_join_game(&mut self, client_id: ClientId, join_params: JoinGameParams) {
        let game_id = join_params.game_id;
        let side = join_params.side;

        // we want the information which side the player joined and the FEN of the joined game
        // to send back to the client.
        let res = (|| {
            let game = self
                .games
                .get_mut(&game_id)
                .ok_or(GameManagerError::GameNotFound(game_id))?;
            game.add_player(client_id, side)
                .map(|side| (side, game.chess.get_fen()))
        })();

        match res {
            Ok((side, fen)) => {
                let response = ServerMessage::GameJoined(game_id, client_id, side, fen);
                if let Some(c) = self.clients.get_mut(&client_id) {
                    let _ = c.tx.send(response).await;
                    c.in_game = Some(game_id);
                }
            }
            Err(_e) => {
                // TODO: Joining game failed; currently we do not propagate a specific event to client here.
                log::warn!(
                    "JoinGame failed for client {} in game {}",
                    client_id,
                    game_id
                );
            }
        }
    }

    async fn handle_leave_game(&mut self, client_id: ClientId) {
        if let Some(c) = self.clients.get_mut(&client_id) {
            if let Some(gid) = c.in_game {
                c.in_game = None;
                if let Some(game) = self.games.get_mut(&gid) {
                    if let Some(side) = game.remove_player(client_id) {
                        let response = ServerMessage::GameLeft(gid, client_id);
                        if let Some(c) = self.clients.get(&client_id) {
                            let _ = c.tx.send(response).await;
                        }
                    }
                }
            }
        }
    }

    /// register a client after a new connection is accepted
    /// TODO: this is quite a dummy as long we don't have persistent accounts.
    /// TODO: later, we probably need to work here when we have real accounts.
    async fn handle_register(&mut self, client_id: ClientId, tx: Sender<ServerMessage>) {
        let client = ClientEndpoint::new(tx);
        self.clients.insert(client_id, client);
    }

    /// Handle a `ChessMove` from a `ClientSession`.
    /// Moves can be accepted (when legal) and rejected (when illegal).
    /// Will also send separate `ServerMessages` for checkmate and stalemate.
    async fn handle_move(&mut self, client_id: ClientId, game_id: GameId, mov: ChessMove) {
        let game = match self.games.get_mut(&game_id) {
            Some(g) => g,
            None => {
                // a move made on a non-existing game
                if let Some(c) = self.clients.get(&client_id) {
                    let _ =
                        c.tx.send(ServerMessage::IllegalMove(ChessError::IllegalMove(mov)))
                            .await;
                }
                return;
            }
        };
        // convert `ChessMove` to SAN notation to send it back to the client.
        let san = mov.to_san(&game.chess);
        let san_len = san.len() as u8;

        match game.make_move(mov, client_id) {
            // a legal move was made and accepted.
            // Update move history and send the updated squares to all clients in the game.
            Ok(changes) => {
                game.move_history.push(mov);

                // convert `Piece` to `WoodPiece`. A `Piece` includes all the server side logic for
                // movement, which the client should not need to know about. A `WoodPiece` is merely
                // the "outer hull", i.e., piece type and color.
                let changes: Vec<(Tile, Option<WoodPiece>)> = changes
                    .iter()
                    .map(|(t, p)| (*t, p.map(|piece| piece.piece)))
                    .collect();

                let clients = game.get_participants();
                for c in &clients {
                    let msg = ServerMessage::MoveAccepted(san_len, san.clone(), changes.clone());
                    if let Some(handler) = self.clients.get(&c) {
                        let _ = handler.tx.send(msg).await;
                    }
                }
                // The move has been executed. Now we check if the game is over,
                // e.g., checkmate or stalemate.
                match game.get_game_state() {
                    ChessGameState::Running => {}
                    ChessGameState::Finished(outcome) => {
                        // Game is finished; save the history in a file, remove it from the game list
                        // and send the outcome to all clients in the game.
                        // Note: we have to remove the game from the list first to get ownership for save_game
                        if let Some(game) = self.games.remove(&game_id) {
                            let _ = self.save_game(&game).await;
                        }
                        match outcome {
                            ChessGameOutcome::Checkmate(is_checkmated) => {
                                for c in &clients {
                                    if let Some(handler) = self.clients.get(&c) {
                                        let _ = handler
                                            .tx
                                            .send(ServerMessage::Checkmate(game_id, is_checkmated))
                                            .await;
                                    }
                                }
                            }
                            ChessGameOutcome::Stalemate => {
                                for c in &clients {
                                    if let Some(handler) = self.clients.get(&c) {
                                        let _ = handler
                                            .tx
                                            .send(ServerMessage::Stalemate(game_id))
                                            .await;
                                    }
                                }
                            }
                            ChessGameOutcome::Resignation(_) => {}
                            ChessGameOutcome::TimeOut(_) => {}
                            ChessGameOutcome::MaterialDraw => {}
                            ChessGameOutcome::TimeOutMaterialDraw => {}
                            ChessGameOutcome::FiftyMoveRule => {}
                        }
                    }
                }
            }

            // The move was illegal and thus rejected
            Err(e) => {
                if let Some(c) = self.clients.get(&client_id) {
                    let _ = c.tx.send(ServerMessage::IllegalMove(e)).await;
                }
            }
        }
    }

    /// The client asked for listing all games.
    async fn handle_query_games(&self, client_id: ClientId) {
        let game_ids: Vec<GameId> = self.games.keys().cloned().collect();
        if let Some(c) = self.clients.get(&client_id) {
            let _ = c.tx.send(ServerMessage::GamesList(game_ids)).await;
        }
    }

    /// The client asked for details of a specific game.
    /// (which players are in it, time settings, etc.)
    async fn handle_query_game_details(&self, client_id: ClientId, game_id: GameId) {
        let game = self.games.get(&game_id);
        match game {
            Some(game) => {
                if let Some(c) = self.clients.get(&client_id) {
                    let _ =
                        c.tx.send(ServerMessage::GameDetails(
                            game_id,
                            game.white_player,
                            game.black_player,
                            game._time,
                            game._time_inc,
                        ))
                        .await;
                }
            }
            None => {
                log::warn!("Client asked for invalid game: {}", game_id)
            }
        }
    }

    /// The client asked for details of another client.
    /// E.g., initially a client only knows other client IDs.
    /// From these IDs, we can look up the name and other client information.
    async fn handle_query_client_details(&self, client_id: ClientId, query_id: ClientId) {
        if let Some(c) = self.clients.get(&client_id) {
            let name = self
                .clients
                .get(&query_id)
                .map(|client| client.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let _ =
                c.tx.send(ServerMessage::ClientDetails(query_id, name))
                    .await;
        }
    }

    /// Setting the nickname of a client.
    async fn handle_set_nickname(&mut self, client_id: ClientId, nickname: String) {
        if let Some(c) = self.clients.get_mut(&client_id) {
            c.name = nickname;
        }
    }

    /// save a game to disk.
    pub async fn save_game(&self, game: &ChessGame) -> std::io::Result<()> {
        let date = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();

        let white = game.white_player.unwrap();
        let white = &self.clients.get(&white).unwrap().name;

        let black = game.black_player.unwrap();
        let black = &self.clients.get(&black).unwrap().name;

        let filename = format!("{}-vs-{}_{}.txt", white, black, date);
        let mut file = File::create(filename).await?;
        for mov in game.move_history.iter() {
            file.write_all(mov.to_string().as_bytes()).await?;
            file.write_all(b"\n").await?;
        }
        file.sync_all().await?;

        Ok(())
    }
}
