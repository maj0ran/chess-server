#[cfg(test)]
use crate::chess::{
    ClientMessage, JoinGameParams, NetMessage, NewGameParams, ServerMessage, UserRoleSelection,
};
#[cfg(test)]
use crate::chess::net::connection::Connection;
#[cfg(test)]
use smol::net::TcpStream;

#[cfg(test)]
pub struct TestClient {
    pub conn: Connection,
}

#[cfg(test)]
impl TestClient {
    pub async fn new(port: u16) -> Self {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        let mut conn = Connection::new(stream);

        // Consume login message
        match conn.read_msg::<ServerMessage>().await {
            Ok(ServerMessage::Login(_)) => {}
            Ok(e) => panic!("Expected Login event, got {:?}", e),
            Err(e) => panic!("Error reading login message: {:?}", e),
        }

        TestClient { conn }
    }

    pub async fn create_game(&mut self, mode: u8, time: u32, time_inc: u32) -> u32 {
        let cmd = ClientMessage::NewGame(NewGameParams {
            mode,
            time,
            time_inc,
        });
        self.conn.write_out(&cmd.to_bytes()).await.unwrap();

        match self.conn.read_msg::<ServerMessage>().await {
            Ok(ServerMessage::GameCreated(game_id, _)) => game_id,
            Ok(e) => panic!("Expected GAME_CREATED response, got {:?}", e),
            Err(e) => panic!("Error reading GAME_CREATED: {:?}", e),
        }
    }

    pub async fn list_games(&mut self) -> Vec<u32> {
        let cmd = ClientMessage::QueryGames;
        self.conn.write_out(&cmd.to_bytes()).await.unwrap();

        loop {
            match self.conn.read_msg::<ServerMessage>().await {
                Ok(ServerMessage::GamesList(game_ids)) => return game_ids,
                Ok(_) => continue,
                Err(e) => panic!("Error reading games list: {:?}", e),
            }
        }
    }

    pub async fn get_game_details(&mut self, game_id: u32) -> ServerMessage {
        let cmd = ClientMessage::QueryGameDetails(game_id);
        self.conn.write_out(&cmd.to_bytes()).await.unwrap();

        match self.conn.read_msg::<ServerMessage>().await {
            Ok(event @ ServerMessage::GameDetails(_, _, _, _, _)) => event,
            Ok(e) => panic!("Expected GameDetails, got {:?}", e),
            Err(e) => panic!("Error reading game details: {:?}", e),
        }
    }

    pub async fn join_game(&mut self, game_id: u32, role: UserRoleSelection) {
        let cmd = ClientMessage::JoinGame(JoinGameParams {
            game_id,
            side: role,
        });
        self.conn.write_out(&cmd.to_bytes()).await.unwrap();

        loop {
            match self.conn.read_msg::<ServerMessage>().await {
                Ok(ServerMessage::GameJoined(_, _, _, _)) => return,
                Ok(_) => {}
                Err(e) => panic!("Error joining game: {:?}", e),
            }
        }
    }

    pub async fn make_move(&mut self, game_id: u32, mov_str: &str) -> ServerMessage {
        let mov = mov_str.parse().unwrap();
        let cmd = ClientMessage::Move(game_id, mov);
        self.conn.write_out(&cmd.to_bytes()).await.unwrap();

        loop {
            match self.conn.read_msg::<ServerMessage>().await {
                Ok(event) => match event {
                    ServerMessage::Update(_) => {
                        // For the purpose of the test_checkmate, we KNOW a CHECKMATE follows d8h4.
                        if mov_str == "d8h4" {
                            if let Ok(next_event) = self.conn.read_msg::<ServerMessage>().await {
                                return next_event;
                            }
                        }
                        return event;
                    }
                    _ => return event,
                },
                Err(e) => panic!("Error reading response to move: {:?}", e),
            }
        }
    }
}
