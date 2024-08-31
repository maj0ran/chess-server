pub mod connection {

    use super::super::buffer::Buffer;
    use crate::net::frame::Frame;

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
    }
}
