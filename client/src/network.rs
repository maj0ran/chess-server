use chess_core::net::connection::Connection;
use chess_core::{ClientMessage, GameId, NetError, NetMessage, NetResult, ServerMessage};
use smol::channel::{Receiver, Sender};
use smol::net::TcpStream;

#[derive(Clone)]
/// `ChessClient` manages the network connection for a chess client.
/// It is designed to be cloned, which allows multiple threads to interact with the same underlying
/// TCP connection concurrently. Cloning `ChessClient` creates a new `Connection` object that
/// shares the same socket handle, as `smol::net::TcpStream` supports concurrent read and write operations.
pub struct NetworkInterface {
    pub cmd_tx: Sender<ClientMessage>,
    pub resp_rx: Receiver<ServerMessage>,
}

impl NetworkInterface {
    pub fn new() -> Self {
        let config = Config::read("settings.cfg");
        Self::with_config(config)
    }

    pub fn with_config(config: Config) -> Self {
        let (cmd_tx, cmd_rx) = smol::channel::unbounded();
        let (resp_tx, resp_rx) = smol::channel::unbounded();
        let server_addr = config.server;

        std::thread::spawn(move || {
            match smol::block_on(network_thread(&server_addr, cmd_rx, resp_tx)) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to start network thread. Server running? {}", e)
                }
            };
        });

        Self { cmd_tx, resp_rx }
    }

    pub fn send(&self, msg: ClientMessage) {
        if let Err(e) = self.cmd_tx.try_send(msg) {
            log::error!("Failed to send message to network thread: {}", e);
        }
    }
}

/// Setting up the network tasks for receiving and transmitting messages.
/// When following the receive/transmit logic, be aware that we have 3 Components:
/// UI <-> Client <-> Server.
/// Transmitting means we receive via channel from the UI and transmit over network to the server.
/// Receiving means we receive via network from the server and transmit over channel to the UI.
/// Thus, cmd_rx is the receiver from the UI and used for the transmitter-client,
/// resp_tx is the sender to the UI and used for the receiver-client.
pub async fn network_thread(
    addr: &str,
    cmd_rx: Receiver<ClientMessage>,
    resp_tx: Sender<ServerMessage>,
) -> NetResult<()> {
    let stream = TcpStream::connect(addr).await?;
    let mut conn = Connection::new(stream);

    log::info!("Network thread started");

    let transmit_conn = conn.clone();
    std::thread::spawn(move || {
        smol::block_on(transmit_thread(transmit_conn, cmd_rx));
    });

    let receive_conn = conn;
    std::thread::spawn(move || {
        smol::block_on(receive_thread(receive_conn, resp_tx));
    });

    Ok(())
}

/// Receives commands from the UI and transmits them to the server
/// cmd_rx it the Receiver channel UI -> Client
pub async fn transmit_thread(mut conn: Connection, cmd_rx: Receiver<ClientMessage>) {
    while let Ok(cmd) = cmd_rx.recv().await {
        log::debug!("Received command from UI: {:?} - sending to server.", cmd);
        if let Err(e) = conn.write_out(&cmd.to_bytes()).await {
            log::error!("Failed to send command to server: {}", e);
        }
    }
    log::info!("Transmit thread shutting down");
}

/// Receive Commands from the server and transmit them to the UI
/// resp_tx is the Sender channel Client -> UI
pub async fn receive_thread(mut conn: Connection, resp_tx: Sender<ServerMessage>) {
    while let Ok(event) = conn.read_msg::<ServerMessage>().await {
        if resp_tx.send(event).await.is_err() {
            log::error!("Failed to send event to UI");
        }
    }
    log::info!("Receive thread shutting down");
}

use crate::config::Config;
use crate::state::{ClientBackend, GameDetails, Overlay, Screen};
use crate::ui::gamelist_menu::UpdateGamesList;
use bevy::prelude::*;
use std::collections::HashMap;

pub fn poll_network(
    mut commands: Commands,
    mut state: ResMut<ClientBackend>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    while let Ok(server_msg) = state.network.resp_rx.try_recv() {
        log::debug!("Received server message: {:?}", server_msg);
        match server_msg {
            ServerMessage::GamesList(games) => {
                let mut games_map: HashMap<GameId, Option<GameDetails>> = HashMap::new();
                for &game_id in &games {
                    log::debug!("Querying for game {}", game_id);
                    state.network.send(ClientMessage::QueryGameDetails(game_id));
                    games_map.insert(game_id, None);
                }
                state.menu_state.games = games_map;
            }
            ServerMessage::GameCreated(id, _) => {
                log::debug!("Game created with ID: {}", id);
                state.network.send(ClientMessage::QueryGames);
            }
            ServerMessage::GameJoined(_, _, _, fen) => {
                state.update_board_from_fen(&fen);
                state.game_state.dirty = true;
                next_screen.set(Screen::Game);
                next_overlay.set(Overlay::None);
            }
            ServerMessage::Update(updates) => {
                for (tile, piece) in updates {
                    if let Some(p) = piece {
                        state.game_state.board.insert(tile.to_string(), p.as_byte());
                    } else {
                        state.game_state.board.remove(&tile.to_string());
                    }
                }
                state.game_state.dirty = true;
            }
            ServerMessage::IllegalMove(err) => {
                log::warn!("{}", err);
                state.menu_state.error_msg = Some(err.to_string());
                // even though the move was illegal, we set the game_state dirty. This is because
                // the user dragged a piece somewhere on the board UI, and we have to
                // let the UI update the board back to its original state.
                state.game_state.dirty = true;
            }
            ServerMessage::Checkmate(_gid, is_checkmated) => {
                state.menu_state.error_msg = Some("Checkmate!".to_string());
                state.game_state.winner = Some(!is_checkmated);
                next_overlay.set(Overlay::GameOver);
                log::info!("Checkmate!");
            }
            ServerMessage::Stalemate(_) => {
                state.menu_state.error_msg = Some("Stalemate!".to_string());
                log::info!("Stalemate!");
            }
            ServerMessage::GameDetails(game_id, white_id, black_id, time, inc) => {
                let game_details = GameDetails {
                    white_player: white_id,
                    black_player: black_id,
                    _time: time,
                    _time_inc: inc,
                };
                if state.menu_state.games.contains_key(&game_id) {
                    state.menu_state.games.insert(game_id, Some(game_details));
                    log::info!("Received game details for game ID: {}", game_id);
                }

                if let Some(wid) = white_id {
                    if !state.menu_state.client_names.contains_key(&wid) {
                        state.network.send(ClientMessage::QueryClientDetails(wid));
                    }
                }
                if let Some(bid) = black_id {
                    if !state.menu_state.client_names.contains_key(&bid) {
                        state.network.send(ClientMessage::QueryClientDetails(bid));
                    }
                }

                commands.trigger(UpdateGamesList);
            }
            ServerMessage::ClientDetails(client_id, name) => {
                state.menu_state.client_names.insert(client_id, name);
                commands.trigger(UpdateGamesList);
            }
            ServerMessage::LoginAccepted(_) => {
                log::info!("Login accepted");
                let name = state.name.clone();
                state.network.send(ClientMessage::SetNickname(name));
            }
            ServerMessage::GameLeft(_gid, _cid) => {}
        }
    }
}
