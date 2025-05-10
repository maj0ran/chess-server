mod testclient;

#[cfg(test)]
pub mod testgames {

    use std::error::Error;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::time::Duration;

    use smol::net::TcpStream;
    use smol::Timer;
    use smol_macros::test;

    use crate::chessmove::{ChessMove, ToChessMove};
    use crate::net::connection::connection::Connection;
    pub struct TestClient {
        conn: Connection,
    }

    trait NetDataStream {
        fn to_bytes(&self) -> Vec<u8>;
    }

    impl NetDataStream for ChessMove {
        fn to_bytes(&self) -> Vec<u8> {
            vec![
                self.src.file as u8,
                self.src.rank as u8,
                self.dst.file as u8,
                self.dst.rank as u8,
            ]
        }
    }

    impl TestClient {
        async fn new() -> TestClient {
            let stream = match TcpStream::connect("127.0.0.1:7878").await {
                Ok(s) => s,
                Err(_) => panic!(),
            };

            let conn = Connection::new(stream);
            TestClient { conn }
        }

        async fn send(&mut self) -> Result<(), Box<dyn Error>> {
            Timer::after(Duration::from_millis(100)).await;

            Ok(())
        }
    }
    test! {
        async fn testgame() {
            env_logger::init();
            let mut client = TestClient::new().await;
            // send new game
            let bytes: [u8; 4] = [10, 65, 32, 1];
            client.conn.write_out(&bytes).await;

            let file = File::open("testgame").unwrap();
            let moves = io::BufReader::new(file).lines();
            for full_line in moves {
                Timer::after(Duration::from_millis(100)).await;
                match full_line {
                    Ok(line) => {
                        let without_comment: Vec<&str> = line.split("#").collect();
                        let without_comment = without_comment.first().unwrap();
                        let testmove: Vec<&str> = without_comment.split(" ").collect();
                        let mov = testmove[0];
                      //  let expected = testmove[1];
                        let chessmove: Option<ChessMove> = mov.to_string().parse();

                        match chessmove {
                            Some(m) => {
                                let data: Vec<u8> = std::iter::once(0xD).chain(m.to_bytes()).collect();
                                log::debug!("sending: {:?}", data);
                                client.conn.write_out(data.as_slice()).await;
                            }
                            None => todo!(),
                        }
                    }
                    Err(_) => todo!(),
                }
            }
            loop {
                let _ = client.send().await;
            }
        }
    }
}
