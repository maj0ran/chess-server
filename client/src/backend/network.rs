use crate::backend::client::*;
use crate::backend::config::*;
use crate::ui::gamelist_menu::UpdateGamesList;

use crate::backend::client::game::{
    ActiveGame, BoardUpdate, GameDetails, GameJoinedEvent, GameOverEvent,
};
use crate::backend::client::lobby::LobbyState;
use bevy::prelude::*;
use chess_core::net::connection::Connection;
use chess_core::protocol::messages::{ClientMessage, ServerMessage};
use chess_core::protocol::parser::NetMessage;
use chess_core::{GameId, NetResult};
use smol::channel::{Receiver, Sender};
use smol::net::TcpStream;
use std::collections::HashMap;

/// `NetTransport` is the interface to networking.
/// it holds the tx/rx channels to communicate from the transmit/receive threads to the main/bevy thread.
#[derive(Resource)]
pub struct NetTransport {
    tx: Sender<ClientMessage>,
    rx: Receiver<ServerMessage>,
}

/// Event that can be used by all parts of the client to send a message to the network thread.
#[derive(Event)]
pub struct NetworkSend(pub ClientMessage);

impl NetTransport {
    pub fn new() -> Self {
        let config = Config::read("settings.cfg");
        Self::with_config(config)
    }

    pub fn with_config(config: Config) -> Self {
        let (tx_to_server, rx_from_client) = smol::channel::unbounded();
        let (tx_to_client, rx_from_server) = smol::channel::unbounded();
        let server_addr = config.server;

        std::thread::spawn(move || {
            match smol::block_on(network_thread(&server_addr, rx_from_client, tx_to_client)) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to start network thread. Server running? {}", e)
                }
            };
        });

        Self {
            tx: tx_to_server,
            rx: rx_from_server,
        }
    }
}

/// Setting up the network tasks for receiving and transmitting messages.
/// When following the receive/transmit logic, be aware that we have 3 Components:
/// Client <-> NetTransport <-> Server.
/// This thread is beste be viewed from the middle perspective, i.e. `NetTransport`:
/// `rx_from_client`: read a message that has been put by the client on `NetTransport`. Gets forwarded to the server.
/// `tx_to_client`: got a message from server, write it on `NetTransport`. Gets forwarded to the client.
pub async fn network_thread(
    addr: &str,
    rx_from_client: Receiver<ClientMessage>,
    tx_to_client: Sender<ServerMessage>,
) -> NetResult<()> {
    let stream = TcpStream::connect(addr).await?;
    let conn = Connection::new(stream);

    log::info!("Network thread started");

    // listens on the `NetTransport` channel and transmits the messages via TCP to the server.
    let mut to_server = conn.clone();
    std::thread::spawn(move || {
        smol::block_on(async move {
            while let Ok(cmd) = rx_from_client.recv().await {
                if let Err(e) = to_server.write_out(&cmd.to_bytes()).await {
                    log::error!("Failed to send command to server: {}", e);
                }
            }
            log::info!("Transmit thread shutting down");
        });
    });

    // listens for TCP messages from the server and transmits them to the `NetTransport` channel.
    let mut from_server = conn;
    std::thread::spawn(move || {
        smol::block_on(async move {
            while let Ok(server_msg) = from_server.read_msg::<ServerMessage>().await {
                if tx_to_client.send(server_msg).await.is_err() {
                    log::error!("Failed to send event to UI");
                }
            }
            log::info!("Receive thread shutting down");
        });
    });

    Ok(())
}

/// The primary network system. It polls `NetTransport` for server messages that have been received
/// by the network thread. This can essentially be seen as the counter-part to `NetTransport.send()`.
/// Here, we are reading whatever the network thread put on `NetTransport`.
///
/// Unlike `send_message()`, this system runs continuously since `ServerMessage`s can come in
/// any time. This is why this is a system in the `FixedUpate`-schedule, weil `send_message() is an observer.
pub fn poll_network(
    mut commands: Commands,
    network: Res<NetTransport>,
    mut lobby: ResMut<LobbyState>,
    active_game: Option<ResMut<ActiveGame>>,
    client_config: Res<ClientConfig>,
) {
    let mut active_game = active_game;
    while let Ok(server_msg) = network.rx.try_recv() {
        log::debug!("Received server message: {:?}", server_msg);

        match server_msg {
            ServerMessage::GamesList(games) => {
                // After receiving a list of games, we instantly ask for the details of each game.
                let mut games_map: HashMap<GameId, Option<GameDetails>> = HashMap::new();
                for &gid in &games {
                    commands.trigger(NetworkSend(ClientMessage::QueryGameDetails(gid)));
                    games_map.insert(gid, None);
                }
                lobby.games = games_map;
            }
            // A new game has been created, we query for a new games list.
            // TODO: could just query the details of the new game and update the internal list.
            ServerMessage::GameCreated(_gid, _cid) => {
                commands.trigger(NetworkSend(ClientMessage::QueryGames));
            }

            // We successfully joined a game, update the chessboard of the game
            ServerMessage::GameJoined(gid, _cid, side, fen) => {
                // HINT: we only receive this message for our own client, not when someone
                // else joined. This is a TODO on the server.
                // once we change the behavior of the server, we also have to add additional
                // logic here to handle the case when someone else joins.
                let mut game = ActiveGame {
                    gid,
                    side,
                    internal_board: HashMap::new(),
                };
                game.update_internal_board_from_fen(&fen);
                commands.insert_resource(game);

                commands.trigger(GameJoinedEvent { gid, side, fen });
            }

            // A piece in the current game has been moved.
            ServerMessage::MoveAccepted(_, _san, updates) => {
                if let Some(game) = active_game.as_mut() {
                    for (tile, piece) in updates {
                        if let Some(p) = piece {
                            game.internal_board.insert(tile.to_string(), p.as_byte());
                        } else {
                            game.internal_board.remove(&tile.to_string());
                        }
                    }
                    commands.trigger(BoardUpdate);
                }
            }

            // Our last move was illegal.
            ServerMessage::IllegalMove(_) => {}

            // We received a game over message.
            ServerMessage::GameOver(_gid, reason) => {
                commands.trigger(GameOverEvent { reason });
            }

            // We received the lobby details of a specific game.
            ServerMessage::GameDetails(gid, white_id, black_id, time, inc) => {
                let game_details = GameDetails {
                    white_player: white_id,
                    black_player: black_id,
                    _time: time,
                    _time_inc: inc,
                };
                if lobby.games.contains_key(&gid) {
                    lobby.games.insert(gid, Some(game_details));
                }
                if let Some(wid) = white_id {
                    if !lobby.client_names.contains_key(&wid) {
                        commands.trigger(NetworkSend(ClientMessage::QueryClientDetails(wid)));
                    }
                }
                if let Some(bid) = black_id {
                    if !lobby.client_names.contains_key(&bid) {
                        commands.trigger(NetworkSend(ClientMessage::QueryClientDetails(bid)));
                    }
                }

                commands.trigger(UpdateGamesList);
            }

            // We received information of another client
            ServerMessage::ClientDetails(cid, name) => {
                lobby.client_names.insert(cid, name);
                commands.trigger(UpdateGamesList);
            }

            // Our Login has been accepted. Send the server our nickname.
            ServerMessage::LoginAccepted(_) => {
                let name = client_config.name.clone();
                commands.trigger(NetworkSend(ClientMessage::SetNickname(name)));
            }

            // We have left the game.
            // TODO: Here and in the server, this should not be only for us but for all clients in a game.
            ServerMessage::GameLeft(_gid, _cid) => {
                commands.remove_resource::<ActiveGame>();
            }
        }
    }
}

/// Put a `ClientMessage` on `NetTransport`. The network thread will automatically
/// forward this message to the external server.
pub fn send_message(ev: On<NetworkSend>, net: Res<NetTransport>) {
    let msg = ev.0.clone();
    if let Err(e) = net.tx.try_send(msg) {
        log::error!("Failed to send message to network thread: {}", e);
    }
}
