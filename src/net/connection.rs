pub mod Connection {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    use crate::net::frame::Frame;

    use super::super::buffer::Buffer;
    use super::super::{Command, NewGame, BUF_LEN};
    pub struct Connection {
        pub buf: Buffer,
        //     pub player: Player, // a connection has a player because the connection is what controls the
        // player
    }

    impl Connection {
        pub fn is_ingame(&self) -> bool {
            // self.player.chess.is_some()
            true
        }
    pub fn create_frame(&self) -> Frame {
        Frame {
            len: self.buf.len as u8,
            content: self.buf.buf,
        }
    }

    }
}
