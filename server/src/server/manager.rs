use crate::chess::chess::Chess;
use crate::chess::san::San;
use crate::server::chessgame::ChessGame;
use chess_core::protocol::messages::{ClientMessage, ServerMessage};
use chess_core::protocol::{JoinGameParams, NewGameParams, UserRoleSelection};
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
        let gid = game.id;
        self.games.insert(gid, game);
        // inform all clients of new game created by $cid
        for c in &self.clients {
            let _ = c.1.tx.send(ServerMessage::GameCreated(gid, cid)).await;
        }
    }

    /// Assign a client to a game and inform it and all other clients about the join.
    async fn handle_join_game(&mut self, cid: ClientId, join_params: JoinGameParams) {
        let gid = join_params.game_id;
        let side = join_params.side;

        match self.add_player_to_game(gid, cid, side) {
            Ok(side) => {
                let msg = ServerMessage::GameJoined(gid, cid, side);
                self.broadcast(gid, msg).await;
            }

            Err(e) => {
                // TODO: Joining game failed; currently we do not propagate a specific event to client here.
                log::warn!("JoinGame failed for client {} in game {}: {}", cid, gid, e);
            }
        }
    }

    async fn handle_leave_game(&mut self, cid: ClientId, gid: GameId) {
        // leave all games that the client is part of. Used for sudden disconnects.
        let mut gids = vec![];
        if gid == 0 {
            for game in self.games.values_mut() {
                if game.get_all_participants().contains(&cid) {
                    gids.push(game.id);
                }
            }
        } else {
            gids.push(gid);
        }
        for gid in gids {
            self.remove_player_from_game(gid, cid);
            let msg = ServerMessage::GameLeft(gid, cid);
            self.broadcast(gid, msg).await;
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
                let msg = ServerMessage::IllegalMove(ChessError::IllegalMove(mov));
                self.send_to(cid, msg).await;
                return;
            }
        };

        // convert `ChessMove` to SAN notation to send it back to the client.
        // we have to make this conversion before make_move() as we need the current board state
        let san = mov.to_san(&game.chess);
        let san_len = san.len() as u8;

        match game.make_move(mov, cid) {
            // a legal move was made and accepted:
            // Update move history, clear any draw offers
            // and send the updated squares to all clients in the game.
            Ok(changes) => {
                game.move_history.push(san.clone());

                game.draw_offer_white = false;
                game.draw_offer_black = false;

                // convert `Piece` to `WoodPiece`. A `Piece` includes all the server side logic for
                // movement, which the client should not need to know about. A `WoodPiece` is merely
                // the "outer hull", i.e., piece type and color.
                let changes: Vec<(Tile, Option<WoodPiece>)> = changes
                    .iter()
                    .map(|(t, p)| (*t, p.map(|piece| piece.piece)))
                    .collect();

                let msg = ServerMessage::MoveAccepted(san_len, san.clone(), changes.clone());
                self.broadcast(gid, msg).await;
                // The move has been executed. Now we check if the game is over,
                // e.g., checkmate or stalemate.
                match self.get_game_state(gid).await {
                    Some(ChessGameState::Running) => {}
                    Some(ChessGameState::Finished(outcome)) => match outcome {
                        reason => {
                            let msg = ServerMessage::GameOver(gid, reason);
                            self.broadcast(gid, msg).await;
                            self.close_game(gid).await;
                        }
                    },
                    None => {
                        log::warn!("Invalid ChessGameState query for game: {}", gid)
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
        let msg = ServerMessage::GamesList(game_ids);
        self.send_to(cid, msg).await;
    }

    /// The client asked for details of a specific game.
    /// (which players are in it, time settings, etc.)
    async fn handle_query_game_details(&self, cid: ClientId, gid: GameId) {
        let game = self.games.get(&gid);
        match game {
            Some(game) => {
                let msg = ServerMessage::GameDetails(
                    gid,
                    game.white_player,
                    game.black_player,
                    game._time,
                    game._time_inc,
                );
                self.send_to(cid, msg).await;
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
        let client_details = self
            .clients
            .get(&query_cid)
            .map(|client| client.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let msg = ServerMessage::ClientDetails(query_cid, client_details);
        self.send_to(cid, msg).await;
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
            let fen = game.chess.get_fen();
            let msg = ServerMessage::BoardState(gid, fen);
            self.send_to(cid, msg).await;
        }
    }

    /// The client asked for the current full move history of a game.
    async fn handle_query_move_history(&self, cid: ClientId, gid: GameId) {
        if let Some(game) = self.games.get(&gid) {
            let msg = ServerMessage::MoveHistory(gid, game.move_history.clone());
            self.send_to(cid, msg).await;
        }
    }

    pub async fn handle_resign(&mut self, cid: ClientId, gid: GameId) {
        let Some(side) = self.get_player_side(gid, cid).await else {
            return;
        };
        let msg = ServerMessage::GameOver(gid, GameOverReason::Resignation(!side));

        self.broadcast(gid, msg).await;
        self.close_game(gid).await;
    }

    pub async fn handle_offer_draw(&mut self, cid: ClientId, gid: GameId) {
        // can only offer draw if both players are in the game
        if !self.is_full(gid) {
            return;
        }

        if self.get_player_white(gid) == Some(cid) {
            self.set_white_draw_offer(gid, true);
        }
        if self.get_player_black(gid) == Some(cid) {
            self.set_black_draw_offer(gid, true)
        }

        // one of the two players offered a draw.
        // we want both players, the opponent, but also the offerer to receive the event
        // that the offer happened successfully.
        if self.get_white_draw_offer(gid) ^ self.get_black_draw_offer(gid) {
            let msg = ServerMessage::DrawOffered(gid);
            self.broadcast(gid, msg).await;
        }

        // both players offered a draw; game is over
        if self.get_white_draw_offer(gid) && self.get_black_draw_offer(gid) {
            let msg = ServerMessage::GameOver(gid, GameOverReason::DrawAgreement);
            self.broadcast(gid, msg).await;
            self.close_game(gid).await;
        }
    }

    fn is_full(&self, gid: GameId) -> bool {
        let game = self.games.get(&gid);
        match game {
            Some(game) => game.get_players().len() == 2,
            None => false,
        }
    }

    fn get_player_white(&self, gid: GameId) -> Option<ClientId> {
        let game = self.games.get(&gid);
        match game {
            Some(game) => game.white_player,
            None => None,
        }
    }

    fn get_player_black(&self, gid: GameId) -> Option<ClientId> {
        let game = self.games.get(&gid);
        match game {
            Some(game) => game.black_player,
            None => None,
        }
    }

    fn get_white_draw_offer(&self, gid: GameId) -> bool {
        let game = self.games.get(&gid);
        match game {
            Some(game) => game.draw_offer_white,
            None => false,
        }
    }

    fn get_black_draw_offer(&self, gid: GameId) -> bool {
        let game = self.games.get(&gid);
        match game {
            Some(game) => game.draw_offer_black,
            None => false,
        }
    }

    fn set_white_draw_offer(&mut self, gid: GameId, draw_offer: bool) {
        let game = self.games.get_mut(&gid);
        match game {
            Some(game) => game.draw_offer_white = draw_offer,
            None => {}
        }
    }

    fn set_black_draw_offer(&mut self, gid: GameId, draw_offer: bool) {
        let game = self.games.get_mut(&gid);
        match game {
            Some(game) => game.draw_offer_black = draw_offer,
            None => {}
        }
    }

    fn add_player_to_game(
        &mut self,
        gid: GameId,
        cid: ClientId,
        side: UserRoleSelection,
    ) -> GameManagerResult<UserRoleSelection> {
        let Some(game) = self.games.get_mut(&gid) else {
            return Err(GameManagerError::GameNotFound(gid));
        };

        game.add_player(cid, side)
    }

    fn remove_player_from_game(&mut self, gid: GameId, cid: ClientId) {
        let Some(game) = self.games.get_mut(&gid) else {
            return;
        };

        game.remove_player(cid);
    }

    /// Get the side of a player in a game.
    /// Returns `None` if the player is a player of the game.
    async fn get_player_side(&self, gid: GameId, cid: ClientId) -> Option<ChessColor> {
        let Some(game) = self.games.get(&gid) else {
            return None;
        };

        game.get_side(cid)
    }

    async fn get_game_state(&self, gid: GameId) -> Option<ChessGameState> {
        let Some(game) = self.games.get(&gid) else {
            return None;
        };

        Some(game.get_game_state())
    }

    /// Broadcast a message to all clients (players and spectators) of a game.
    async fn broadcast(&mut self, gid: GameId, message: ServerMessage) {
        if let Some(game) = self.games.get(&gid) {
            for c in game.get_all_participants() {
                let _ = self.clients.get(&c).unwrap().tx.send(message.clone()).await;
            }
        }
    }

    async fn send_to(&self, cid: ClientId, message: ServerMessage) {
        if let Some(client) = self.clients.get(&cid) {
            let _ = client.tx.send(message).await;
        }
    }

    /// Remove game from `GameManager` and save game history to disk.
    async fn close_game(&mut self, gid: GameId) {
        if let Some(game) = self.games.remove(&gid) {
            let _ = self.save_game(&game).await;
        }
    }

    /// save game history to disk.
    async fn save_game(&self, game: &ChessGame) -> std::io::Result<()> {
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
