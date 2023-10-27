use super::*;

use tokio::net::TcpListener;

pub struct Server {
    _listener: Option<TcpListener>,
}

impl Server {
    pub fn new() -> Server {
        Server { _listener: None }
    }

    pub async fn listen(&self) -> Result<()> {
        info!("Listening...");
        let listener = TcpListener::bind("127.0.0.1:7878").await?;

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("got connection from {}!", addr);
            let hndl = Connection {
                buf: Buffer::new(socket),
            };

            let mut client = Client::new("Marian".to_string(), hndl);
            tokio::spawn(async move {
                client.run().await;
            });
        }
    }
}
