#[cfg(test)]
pub mod testgames {

    use std::error::Error;
    use std::time::Duration;
    use tokio::time::sleep;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
    };

    use crate::net::buffer::Buffer;
    use crate::net::frame::Frame;
    pub struct TestClient {
        buffer: Buffer,
    }

    impl TestClient {
        async fn new() -> TestClient {
            let stream = match TcpStream::connect("127.0.0.1:7878").await {
                Ok(s) => s,
                Err(_) => panic!(),
            };

            let buffer = Buffer::new(stream);
            TestClient { buffer }
        }

        async fn send(&mut self) -> Result<(), Box<dyn Error>> {
            let bytes: [u8; 5] = [10, 32, 65, 32, 1];
            let len = bytes.len() as u8;
            sleep(Duration::from_millis(100)).await;
            let frame = Frame {
                len,
                content: self.buffer.buf,
            };
            self.buffer.write(frame).await;
            Ok(())
        }
    }

    #[tokio::test]
    async fn testgame() {
        let mut client = TestClient::new().await;

        loop {
            let _ = client.send().await;
        }
    }
}
