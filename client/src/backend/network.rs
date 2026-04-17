use crate::backend::client::{
    BoardUpdate, ClientBackend, GameDetails, GameJoinedEvent, GameOverEvent, GameState,
};
use crate::backend::config::Config;
use crate::ui::gamelist_menu::UpdateGamesList;
use crate::ui::views::gameview::chessboard::RequestMove;
use crate::ui::views::gameview::chessboard::board::ResetSelection;

use bevy::prelude::*;
use chess_core::net::connection::Connection;
use chess_core::protocol::messages::{ClientMessage, ServerMessage};
use chess_core::protocol::parser::NetMessage;
use chess_core::{ChessMove, GameId, NetResult, Tile};
use smol::channel::{Receiver, Sender};
use smol::net::TcpStream;
use std::collections::HashMap;

/// `NetworkInterface` is the tunnel to networking.
/// it holds the tx/rx channels to communicate from the transmit/receive threads to the main/bevy thread.
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
    let conn = Connection::new(stream);

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
        if let Err(e) = conn.write_out(&cmd.to_bytes()).await {
            log::error!("Failed to send command to server: {}", e);
        }
    }
    log::info!("Transmit thread shutting down");
}

/// Receive `ServerMessage` from the server and forward it to the network system (poll_network).
/// `msg_tx` is the Sender channel to the main thread.
pub async fn receive_thread(mut conn: Connection, msg_tx: Sender<ServerMessage>) {
    while let Ok(server_msg) = conn.read_msg::<ServerMessage>().await {
        if msg_tx.send(server_msg).await.is_err() {
            log::error!("Failed to send event to UI");
        }
    }
    log::info!("Receive thread shutting down");
}

/// The network main thread that receives messages from the receive-thread and reacts by sending
/// events to the UI.
/// This is currently a bevy system in the `Update` schedule and hence called every tick
/// to check for new messages from the server. An event-based system might be worth considering...
pub fn poll_network(mut commands: Commands, mut state: ResMut<ClientBackend>) {
    while let Ok(server_msg) = state.network.resp_rx.try_recv() {
        log::debug!("Received server message: {:?}", server_msg);

        match server_msg {
            ServerMessage::GamesList(games) => {
                // After receiving a list of games, we instantly ask for the details of each game.
                let mut games_map: HashMap<GameId, Option<GameDetails>> = HashMap::new();
                for &gid in &games {
                    state.network.send(ClientMessage::QueryGameDetails(gid));
                    games_map.insert(gid, None);
                }
                state.menu_state.games = games_map;
            }

            ServerMessage::GameCreated(_gid, _cid) => {
                state.network.send(ClientMessage::QueryGames);
            }

            ServerMessage::GameJoined(gid, _cid, side, fen) => {
                // HINT: we only receive this message for our own client, not when someone
                // else joined. This is a TODO on the server.
                // once we change the behavior of the server, we also have to add additional
                // logic here to handle the case when someone else joins.
                let game_state = GameState {
                    gid,
                    side,
                    internal_board: HashMap::new(),
                };
                state.game_state = Some(game_state);

                commands.trigger(GameJoinedEvent { gid, side, fen });
            }

            ServerMessage::MoveAccepted(_, _san, updates) => {
                if let Some(game_state) = state.game_state.as_mut() {
                    for (tile, piece) in updates {
                        if let Some(p) = piece {
                            game_state
                                .internal_board
                                .insert(tile.to_string(), p.as_byte());
                        } else {
                            game_state.internal_board.remove(&tile.to_string());
                        }
                    }
                    commands.trigger(BoardUpdate);
                }
            }

            ServerMessage::IllegalMove(_) => {}

            ServerMessage::GameOver(_gid, reason) => {
                commands.trigger(GameOverEvent { reason });
            }

            ServerMessage::GameDetails(gid, white_id, black_id, time, inc) => {
                let game_details = GameDetails {
                    white_player: white_id,
                    black_player: black_id,
                    _time: time,
                    _time_inc: inc,
                };
                if state.menu_state.games.contains_key(&gid) {
                    state.menu_state.games.insert(gid, Some(game_details));
                }
                // From GameDetails, we received the client IDs for white and black player.
                // Query those clients to receive their names.
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

            ServerMessage::ClientDetails(cid, name) => {
                // when we receive ClientDetails, we store those details locally.
                state.menu_state.client_names.insert(cid, name);
                commands.trigger(UpdateGamesList);
            }

            ServerMessage::LoginAccepted(_) => {
                let name = state.name.clone();
                state.network.send(ClientMessage::SetNickname(name));
            }

            ServerMessage::GameLeft(_gid, _cid) => {
                state.game_state = None;
                commands.trigger(UpdateGamesList);
            }
        }
    }
}

pub fn on_move_request(ev: On<RequestMove>, mut commands: Commands, backend: Res<ClientBackend>) {
    let src = Tile::from(ev.source.as_str());
    let dst = Tile::from(ev.destination.as_str());
    let promotion = ev.promotion;
    let game_id = backend.game_state.as_ref().unwrap().gid;

    backend.network.send(ClientMessage::Move(
        game_id,
        ChessMove {
            src,
            dst,
            special: promotion,
        },
    ));
    commands.trigger(ResetSelection);
}
