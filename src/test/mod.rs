#[cfg(test)]
pub mod testgames {

    use log::debug;
    use std::error::Error;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::time::Duration;
    use tokio::time::sleep;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    use crate::chessmove::{ChessMove, ToChessMove};
    use crate::net::buffer::Buffer;
    use crate::net::frame::Frame;
    pub struct TestClient {
        stream: TcpStream,
        buffer: Buffer,
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

            let buffer = Buffer::new();
            TestClient { stream, buffer }
        }

        async fn send(&mut self) -> Result<(), Box<dyn Error>> {
            sleep(Duration::from_millis(100)).await;

            Ok(())
        }
    }

    #[tokio::test]
    async fn testgame() {
        env_logger::init();
        let mut client = TestClient::new().await;
        // send new game
        let bytes: [u8; 5] = [10, 32, 65, 32, 1];
        client.buffer.write_frame(&bytes);
        client.buffer.write(&mut client.stream).await;

        let file = File::open("testgame").unwrap();
        let moves = io::BufReader::new(file).lines();
        for full_line in moves {
            sleep(Duration::from_millis(100)).await;
            match full_line {
                Ok(line) => {
                    let without_comment: Vec<&str> = line.split("#").collect();
                    let without_comment = without_comment.first().unwrap();
                    let testmove: Vec<&str> = without_comment.split(" ").collect();
                    let mov = testmove[0];
                    let expected = testmove[1];
                    let chessmove: Option<ChessMove> = mov.to_string().parse();

                    match chessmove {
                        Some(m) => {
                            let data = m.to_bytes();
                            debug!("sending: {:?}", data);
                            client.buffer.write_frame(data.as_slice());
                            client.buffer.write(&mut client.stream).await;
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
