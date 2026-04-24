use crate::client::session::*;
use crate::ui::gamelist_menu::UpdateGamesList;

use crate::client::game::{ActiveGame, BoardUpdate, GameDetails, GameJoinedEvent, GameOverEvent};
use crate::client::lobby::LobbyState;
use crate::ui::views::gameview::game_screen::DrawOffered;
use crate::ui::views::gameview::historypanel::movehistory::{
    MoveHistoryFullRefresh, MoveHistoryUpdated,
};
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
/// This is basically the mediator between the client and the network thread. On the one side,
/// `network_thread()` is using this resource to put messages from the remote server into it and receive
/// messages from the client from it to send them to the remote server.
/// On the other side, the client systems are using this resource to send messages to the network thread.
#[derive(Resource)]
pub struct NetTransport {
    tx: Sender<ClientMessage>,
    rx: Receiver<ServerMessage>,
}

impl NetTransport {
    pub fn new(server_addr: String) -> Self {
        let (tx_to_server, rx_from_client) = smol::channel::unbounded();
        let (tx_to_client, rx_from_server) = smol::channel::unbounded();

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
    mut session: ResMut<ClientSession>,
) {
    let mut active_game = active_game;
    while let Ok(server_msg) = network.rx.try_recv() {
        log::debug!("Received server message: {:?}", server_msg);

        match server_msg {
            /* Receive a list of all available games. */
            ServerMessage::GamesList(mut games) => {
                lobby.clear_games();
                // After receiving a list of games, we instantly ask for the details of each game.
                for &mut gid in &mut games {
                    commands.trigger(NetworkSend(ClientMessage::QueryGameDetails(gid)));
                }
            }

            /* We received the lobby details of a specific game. */
            ServerMessage::GameDetails(gid, white_id, black_id, time, inc) => {
                let game_details = GameDetails {
                    white_player: white_id,
                    black_player: black_id,
                    _time: time,
                    _time_inc: inc,
                };
                lobby.update_game_info(gid, game_details);

                if let Some(ref mut active_game) = active_game {
                    if active_game.gid == gid {
                        active_game.game_info = game_details;
                    }
                }

                if let Some(wid) = white_id {
                    if !lobby.has_client_info(wid) {
                        commands.trigger(NetworkSend(ClientMessage::QueryClientDetails(wid)));
                    }
                }
                if let Some(bid) = black_id {
                    if !lobby.has_client_info(bid) {
                        commands.trigger(NetworkSend(ClientMessage::QueryClientDetails(bid)));
                    }
                }
                commands.trigger(UpdateGamesList);
            }

            /* We received information of another client */
            ServerMessage::ClientDetails(cid, name) => {
                lobby.update_client_info(cid, name);
                commands.trigger(UpdateGamesList);
            }

            /* A new game has been created, we query for a new games list. */
            // TODO: could just query the details of the new game and update the internal list.
            ServerMessage::GameCreated(_gid, _cid) => {
                commands.trigger(NetworkSend(ClientMessage::QueryGames));
            }

            /* Someone (or we) successfully joined a game. */
            ServerMessage::GameJoined(gid, cid, side) => {
                // the join changed the game details, refresh it
                commands.trigger(NetworkSend(ClientMessage::QueryGameDetails(gid)));

                // it is us, so set the active game
                if session.id == Some(cid) {
                    let game_info = lobby.get_game_info(gid).copied().unwrap();
                    let game = ActiveGame {
                        gid,
                        side,
                        internal_board: HashMap::new(),
                        game_info,

                        move_history: Vec::new(),
                    };
                    commands.insert_resource(game);
                    // send event to the UI to trigger the switch to the game screen, query game info
                    commands.trigger(GameJoinedEvent { gid, side });
                    commands.trigger(NetworkSend(ClientMessage::QueryBoard(gid)));
                    commands.trigger(NetworkSend(ClientMessage::QueryMoveHistory(gid)));
                }
            }

            /* A piece in the current game has been moved. */
            ServerMessage::MoveAccepted(_, san, updates) => {
                if let Some(game) = active_game.as_mut() {
                    game.move_history.push(san.clone());

                    for (tile, piece) in updates {
                        if let Some(p) = piece {
                            game.internal_board.insert(tile.to_string(), p.as_byte());
                        } else {
                            game.internal_board.remove(&tile.to_string());
                        }
                    }
                    commands.trigger(BoardUpdate);
                    commands.trigger(MoveHistoryUpdated);
                    commands.trigger(DrawOffered(false)); // Reset any draw offer
                }
            }

            /* Our last move was illegal. */
            ServerMessage::IllegalMove(_) => {}

            /* We received a game over message. */
            ServerMessage::GameOver(_gid, reason) => {
                commands.trigger(GameOverEvent { reason });
            }

            /* Our Login has been accepted. Send the server our nickname. */
            ServerMessage::LoginAccepted(cid) => {
                session.id = Some(cid);
                let name = session.name.clone();

                log::info!("Assigned session id: {}", cid);

                commands.trigger(NetworkSend(ClientMessage::SetNickname(name)));
            }

            /* Someone (or we) left the game. */
            ServerMessage::GameLeft(gid, cid) => {
                if Some(cid) == session.id {
                    commands.remove_resource::<ActiveGame>();
                } else {
                    commands.trigger(NetworkSend(ClientMessage::QueryGameDetails(gid)));
                }
            }

            /* We received the board state (FEN). */
            ServerMessage::BoardState(gid, fen) => {
                if let Some(game) = active_game.as_mut() {
                    if game.gid == gid {
                        game.update_internal_board_from_fen(&fen);
                        commands.trigger(BoardUpdate);
                    }
                }
            }

            /* We received the move history of a game */
            ServerMessage::MoveHistory(gid, history) => {
                if let Some(game) = active_game.as_mut() {
                    if game.gid == gid {
                        game.move_history = history;
                        commands.trigger(MoveHistoryFullRefresh);
                    }
                }
            }

            /* We received a draw offer from a player */
            ServerMessage::DrawOffered(gid) => {
                if let Some(game) = &active_game {
                    if gid == game.gid {
                        commands.trigger(DrawOffered(true));
                    }
                }
            }
        }
    }
}

/// Event that can be used by all parts of the client to send a message to the network thread.
#[derive(Event)]
pub struct NetworkSend(pub ClientMessage);

/// Put a `ClientMessage` on `NetTransport`. The network thread will automatically
/// forward this message to the external server.
pub fn send_message(ev: On<NetworkSend>, net: Res<NetTransport>) {
    let msg = ev.0.clone();
    if let Err(e) = net.tx.try_send(msg) {
        log::error!("Failed to send message to network thread: {}", e);
    }
}
