use crate::chess::chess::Chess;
use crate::server::chessgame::{ChessGame, ChessGameState};
use chess_core::*;
use smol::channel::{Receiver, Sender};
use std::collections::HashMap;

/// The endpoint of a client for the `GameManager`.
/// Those are used by the `GameManager` to keep a connection
/// to all `ClientSession` via channels.
/// Since the clients all run in their own task, they cannot be
/// directly accessed by the `GameManager`. So we use channels
/// to let messages flow from the `GameManager` task to the
/// individual `ClientSession` tasks.
pub struct ClientEndpoint {
    pub id: ClientId,
    pub tx: Sender<ServerMessage>,
}

impl ClientEndpoint {
    pub fn new(tx: Sender<ServerMessage>) -> Self {
        ClientEndpoint { id: 1, tx }
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
                        ClientMessage::LeaveGame(game_id) => {
                            self.handle_leave_game(client_id, game_id).await;
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
                if let Some(c) = self.clients.get(&client_id) {
                    let _ = c.tx.send(response).await;
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

    async fn handle_leave_game(&mut self, client_id: ClientId, game_id: GameId) {
        if let Some(game) = self.games.get_mut(&game_id) {
            if let Some(side) = game.remove_player(client_id) {
                let response = ServerMessage::GameLeft(game_id, client_id);
                if let Some(c) = self.clients.get(&client_id) {
                    let _ = c.tx.send(response).await;
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
        match game.make_move(mov, client_id) {
            // a legal move was made and accepted.
            // Send the updates squares to all clients in the game.
            Ok(changes) => {
                let clients = game.get_participants();
                for c in &clients {
                    let client_changes: Vec<(Tile, Option<WoodPiece>)> = changes
                        .iter()
                        .map(|(t, p)| (*t, p.map(|piece| piece.piece)))
                        .collect();
                    let msg = ServerMessage::Update(client_changes);
                    if let Some(handler) = self.clients.get(&c) {
                        let _ = handler.tx.send(msg).await;
                    }
                }
                // The move has been executed. Now we check if the game is over,
                // e.g., checkmate or stalemate.
                match game.get_game_state() {
                    ChessGameState::Running => {}
                    ChessGameState::Checkmate(is_checkmated) => {
                        for c in &clients {
                            if let Some(handler) = self.clients.get(&c) {
                                let _ = handler
                                    .tx
                                    .send(ServerMessage::Checkmate(game_id, is_checkmated))
                                    .await;
                            }
                        }
                    }
                    ChessGameState::Stalemate => {
                        for c in &clients {
                            if let Some(handler) = self.clients.get(&c) {
                                let _ = handler.tx.send(ServerMessage::Stalemate(game_id)).await;
                            }
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
}
