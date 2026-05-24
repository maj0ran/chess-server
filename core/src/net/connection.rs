use crate::net::buffer::Buffer;
use crate::protocol::parser::NetMessage;
use crate::NetError;
use crate::NetResult;
use smol::io::AsyncReadExt;
use smol::io::AsyncWriteExt;
use smol::io::{AsyncRead, AsyncWrite};
use smol::io::{ReadHalf, WriteHalf};

/// A `Connection` provides a generic interface for communicating over a TCP stream.
/// It can be used both on the server and client sides to send and receive messages
/// that implement the `NetMessage` trait. Those messages will be parsed into byte stream
/// that can be sent over the network.
pub struct Connection<S> {
    /// `stream` is either a TCP Stream or a TLS stream. Since we are only using TLS, we could
    /// discard this use of Generics, but our code base once changed from TCP to TLS and this
    /// shows best that things are not much different.
    pub stream: S,
    pub buf: Buffer,
}

impl<S> Connection<S> {
    pub fn new(socket: S) -> Connection<S> {
        Connection {
            stream: socket,
            buf: Buffer::new(),
        }
    }
}

impl<S: AsyncRead + Unpin> Connection<S> {
    /// Reads a frame from the stream and parses it into a message of type `T`.
    /// That is, either a `ClientMessage` or a `ServerMessage`.
    /// Returns `NetResult<T>` if successful, or an error otherwise.
    pub async fn read_msg<T: NetMessage>(&mut self) -> NetResult<T> {
        // read the first byte which indicates the length.
        // this value will be discarded and not be part of the read buffer
        let mut len = [0u8; 2];
        self.stream.read_exact(&mut len).await?;
        let length = u16::from_le_bytes(len);
        if length == 0 {
            return Err(NetError::Protocol("received zero-length frame".to_string()));
        }
        if length as usize > Buffer::BUF_LEN {
            return Err(NetError::Protocol(format!(
                "message-length too big!: {} bytes",
                length
            )));
        }

        // read {length} bytes from stream and write the actual message into our buffer
        self.stream
            .read_exact(&mut self.buf[..length as usize])
            .await?;
        self.buf.len = length as usize;
        log::trace!("received frame!: {} (Length: {})", &self.buf, self.buf.len);

        T::from_bytes(&self.buf[..self.buf.len])
    }
}

impl<S: AsyncWrite + Unpin> Connection<S> {
    /// Writes a message to the stream.
    /// The message is serialized into a byte stream and written to the stream.
    /// Returns `Ok(())` if the message was successfully written, or a `NetError` otherwise.
    pub async fn write_out(&mut self, data: &[u8]) -> NetResult<()> {
        if data.len() > Buffer::BUF_LEN {
            return Err(NetError::Protocol(format!(
                "message too long for protocol: {} bytes",
                data.len()
            )));
        }

        let len = data.len() as u16;
        let mut buf = vec![];
        buf.extend_from_slice(len.to_le_bytes().as_slice());
        buf.extend_from_slice(data);

        self.stream.write_all(&buf).await?;
        log::trace!("sent buffer: {:?} ({} bytes)", buf, data.len());
        Ok(())
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    pub fn split(self) -> (Connection<ReadHalf<S>>, Connection<WriteHalf<S>>) {
        let (r, w) = smol::io::split(self.stream);
        (Connection::new(r), Connection::new(w))
    }
}
