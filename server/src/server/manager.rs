use crate::chess::chess::Chess;
use crate::chess::san::San;
use crate::server::chessgame::ChessGame;
use chess_core::protocol::messages::{ClientMessage, ServerMessage};
use chess_core::protocol::{JoinGameParams, NewGameParams};
use chess_core::states::{ChessGameState, GameOverReason};
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
}

impl ClientEndpoint {
    pub fn new(tx: Sender<ServerMessage>) -> Self {
        ClientEndpoint {
            tx,
            name: String::from(""),
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
            next_game_id: 1,
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
            draw_offer_white: false,
            draw_offer_black: false,
            move_history: vec![],
        }
    }

    /// The main loop of the `GameManager`.
    /// Here, the GM listens for `ClientMessages` from `ClientSessions` on its receiver channel.
    /// It will then process the message and send back `ServerMessages` to the `ClientSessions`.
    pub async fn run(&mut self) {
        loop {
            match self.rx.recv().await {
                Ok((cid, cmd)) => {
                    // the actual command
                    match cmd {
                        ClientMessage::NewGame(game_params) => {
                            self.handle_new_game(cid, game_params).await;
                        }
                        ClientMessage::JoinGame(join_params) => {
                            self.handle_join_game(cid, join_params).await;
                        }
                        ClientMessage::Register(tx) => {
                            self.handle_register(cid, tx).await;
                        }
                        ClientMessage::Move(gid, mov) => {
                            self.handle_move(cid, gid, mov).await;
                        }
                        ClientMessage::QueryGames => {
                            self.handle_query_games(cid).await;
                        }
                        ClientMessage::QueryGameDetails(gid) => {
                            self.handle_query_game_details(cid, gid).await;
                        }
                        ClientMessage::QueryClientDetails(cid_query) => {
                            self.handle_query_client_details(cid, cid_query).await;
                        }
                        ClientMessage::LeaveGame(gid) => {
                            self.handle_leave_game(cid, gid).await;
                        }
                        ClientMessage::SetNickname(name) => {
                            log::info!("Set nickname for client {} to {}", cid, name);
                            self.handle_set_nickname(cid, name).await;
                        }
                        ClientMessage::QueryBoard(gid) => {
                            self.handle_query_board(cid, gid).await;
                        }
                        ClientMessage::QueryMoveHistory(gid) => {
                            self.handle_query_move_history(cid, gid).await;
                        }
                        ClientMessage::Resign(gid) => {
                            self.handle_resign(cid, gid).await;
                        }
                        ClientMessage::OfferDraw(gid) => {
                            self.handle_offer_draw(cid, gid).await;
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
    async fn handle_new_game(&mut self, cid: ClientId, game_params: NewGameParams) {
        let game = self.create_game(game_params);
        let id = game.id;
        self.games.insert(id, game);
        // inform all clients of new game
        for c in &self.clients {
            let _ = c.1.tx.send(ServerMessage::GameCreated(id, cid)).await;
        }
    }

    /// Assign a client to a game and inform the `ClientSession` about it.
    /// TODO: should inform all clients that are connected to the game.
    async fn handle_join_game(&mut self, cid: ClientId, join_params: JoinGameParams) {
        let gid = join_params.game_id;
        let side = join_params.side;

        // get the game and add the player
        let game = if let Some(game) = self.games.get_mut(&gid) {
            game
        } else {
            log::warn!("JoinGame failed for client {} in game {}", cid, gid);
            return;
        };

        match game.add_player(cid, side).map(|side| side) {
            Ok(side) => {
                let clients = game.get_participants();
                for c in &clients {
                    let msg = ServerMessage::GameJoined(gid, cid, side);
                    if let Some(handler) = self.clients.get(&c) {
                        let _ = handler.tx.send(msg).await;
                    }
                }
            }

            Err(_e) => {
                // TODO: Joining game failed; currently we do not propagate a specific event to client here.
                log::warn!("JoinGame failed for client {} in game {}", cid, gid);
            }
        }
    }

    async fn handle_leave_game(&mut self, cid: ClientId, gid: GameId) {
        if gid == 0 {
            for game in self.games.values_mut() {
                if game.get_participants().contains(&cid) {
                    game.remove_player(cid);
                }
                let clients = game.get_participants();
                for c in &clients {
                    let response = ServerMessage::GameLeft(game.id, cid);
                    if let Some(c) = self.clients.get(&c) {
                        let _ = c.tx.send(response).await;
                    }
                }
            }
        }
        if let Some(game) = self.games.get_mut(&gid) {
            let clients = game.get_participants();
            for c in &clients {
                let response = ServerMessage::GameLeft(gid, cid);
                if let Some(c) = self.clients.get(&c) {
                    let _ = c.tx.send(response).await;
                }
            }
            let _ = game.remove_player(cid);
        }
    }

    /// register a client after a new connection is accepted
    /// TODO: this is quite a dummy as long we don't have persistent accounts.
    /// TODO: later, we probably need to work here when we have real accounts.
    async fn handle_register(&mut self, cid: ClientId, tx: Sender<ServerMessage>) {
        let client = ClientEndpoint::new(tx);
        self.clients.insert(cid, client);
    }

    /// Handle a `ChessMove` from a `ClientSession`.
    /// Moves can be accepted (when legal) and rejected (when illegal).
    /// Will also send separate `ServerMessages` for checkmate and stalemate.
    async fn handle_move(&mut self, cid: ClientId, gid: GameId, mov: ChessMove) {
        let game = match self.games.get_mut(&gid) {
            Some(g) => g,
            None => {
                // a move made on a non-existing game
                if let Some(c) = self.clients.get(&cid) {
                    let _ =
                        c.tx.send(ServerMessage::IllegalMove(ChessError::IllegalMove(mov)))
                            .await;
                }
                return;
            }
        };

        // convert `ChessMove` to SAN notation to send it back to the client.
        // we have to make this conversion before make_move() as we need the current board state
        let san = mov.to_san(&game.chess);
        let san_len = san.len() as u8;

        match game.make_move(mov, cid) {
            // a legal move was made and accepted.
            // Update move history and send the updated squares to all clients in the game.
            Ok(changes) => {
                game.move_history.push(san.clone());

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
                        if let Some(game) = self.games.remove(&gid) {
                            let _ = self.save_game(&game).await;
                        }
                        match outcome {
                            reason => {
                                for c in &clients {
                                    if let Some(handler) = self.clients.get(&c) {
                                        let _ = handler
                                            .tx
                                            .send(ServerMessage::GameOver(gid, reason))
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // The move was illegal and thus rejected
            Err(e) => {
                if let Some(c) = self.clients.get(&cid) {
                    let _ = c.tx.send(ServerMessage::IllegalMove(e)).await;
                }
            }
        }
    }

    /// The client asked for listing all games.
    async fn handle_query_games(&self, cid: ClientId) {
        let game_ids: Vec<GameId> = self.games.keys().cloned().collect();
        if let Some(c) = self.clients.get(&cid) {
            let _ = c.tx.send(ServerMessage::GamesList(game_ids)).await;
        }
    }

    /// The client asked for details of a specific game.
    /// (which players are in it, time settings, etc.)
    async fn handle_query_game_details(&self, cid: ClientId, gid: GameId) {
        let game = self.games.get(&gid);
        match game {
            Some(game) => {
                if let Some(c) = self.clients.get(&cid) {
                    let _ =
                        c.tx.send(ServerMessage::GameDetails(
                            gid,
                            game.white_player,
                            game.black_player,
                            game._time,
                            game._time_inc,
                        ))
                        .await;
                }
            }
            None => {
                log::warn!("Client asked for invalid game: {}", gid)
            }
        }
    }

    /// The client asked for details of another client.
    /// E.g., initially a client only knows other client IDs.
    /// From these IDs, we can look up the name and other client information.
    async fn handle_query_client_details(&self, cid: ClientId, query_cid: ClientId) {
        if let Some(c) = self.clients.get(&cid) {
            let name = self
                .clients
                .get(&query_cid)
                .map(|client| client.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let _ =
                c.tx.send(ServerMessage::ClientDetails(query_cid, name))
                    .await;
        }
    }

    /// Setting the nickname of a client.
    async fn handle_set_nickname(&mut self, cid: ClientId, nickname: String) {
        if let Some(c) = self.clients.get_mut(&cid) {
            c.name = nickname;
        }
    }

    /// The client asked for the current board state. (FEN)
    async fn handle_query_board(&self, cid: ClientId, gid: GameId) {
        if let Some(game) = self.games.get(&gid) {
            if let Some(c) = self.clients.get(&cid) {
                let fen = game.chess.get_fen();
                let _ = c.tx.send(ServerMessage::BoardState(gid, fen)).await;
            }
        }
    }

    /// The client asked for the current full move history of a game.
    async fn handle_query_move_history(&self, cid: ClientId, gid: GameId) {
        if let Some(game) = self.games.get(&gid) {
            if let Some(c) = self.clients.get(&cid) {
                let _ =
                    c.tx.send(ServerMessage::MoveHistory(gid, game.move_history.clone()))
                        .await;
            }
        }
    }

    pub async fn handle_resign(&mut self, cid: ClientId, gid: GameId) {
        let game = if let Some(game) = self.games.remove(&gid) {
            let _ = self.save_game(&game).await;
            game
        } else {
            return;
        };

        let side = if let Some(s) = game.get_side(cid) {
            s
        } else {
            return;
        };

        for c in game.get_participants() {
            let _ = self
                .clients
                .get(&c)
                .unwrap()
                .tx
                .send(ServerMessage::GameOver(
                    gid,
                    GameOverReason::Resignation(!side),
                ))
                .await;
        }
    }

    pub async fn handle_offer_draw(&mut self, cid: ClientId, gid: GameId) {
        let game = if let Some(game) = self.games.get_mut(&gid) {
            game
        } else {
            return;
        };

        if game.white_player == Some(cid) {
            game.draw_offer_white = true;
        }
        if game.black_player == Some(cid) {
            game.draw_offer_black = true;
        }

        // one of the two players offered a draw; sent offer to other
        if game.draw_offer_white ^ game.draw_offer_black {
            if let Some(opponent) = game.get_opponent(cid) {
                let _ = self
                    .clients
                    .get(&opponent)
                    .unwrap()
                    .tx
                    .send(ServerMessage::DrawOffered(gid))
                    .await;
            }
        }

        // both players offered a draw; game is over
        if game.draw_offer_white && game.draw_offer_black {
            for c in game.get_participants() {
                let _ = self
                    .clients
                    .get(&c)
                    .unwrap()
                    .tx
                    .send(ServerMessage::GameOver(gid, GameOverReason::DrawAgreement))
                    .await;
            }
            let game = self.games.remove(&gid).unwrap();
            let _ = self.save_game(&game).await;
        }
    }

    /// save a game to disk.
    pub async fn save_game(&self, game: &ChessGame) -> std::io::Result<()> {
        let date = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();

        let white = game.white_player.map_or("Unknown".to_string(), |p| {
            self.clients.get(&p).unwrap().name.clone()
        });
        let black = game.black_player.map_or("Unknown".to_string(), |p| {
            self.clients.get(&p).unwrap().name.clone()
        });

        let filename = format!("{}-vs-{}_{}.txt", white, black, date);
        let mut file = File::create(filename).await?;

        let mut fullmove = false;
        for mov in game.move_history.iter() {
            file.write_all(mov.to_string().as_bytes()).await?;
            if !fullmove {
                file.write_all(b" ").await?;
                fullmove = true;
            } else {
                file.write_all(b"\n").await?;
                fullmove = false;
            }
        }
        file.sync_all().await?;

        Ok(())
    }
}
