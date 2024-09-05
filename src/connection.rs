use std::io::Cursor;

use bytes::{Buf, BytesMut};
use num_derive::FromPrimitive;
use thiserror::Error;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::JoinHandle,
};

use crate::packet::{Packet, PacketCheckOutcome, PacketDecodeError};

pub struct ConnectionManager {
    tcp_listener: TcpListener,
}

impl ConnectionManager {
    pub async fn new<A>(address: A) -> std::io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        Ok(Self {
            tcp_listener: TcpListener::bind(address).await?,
        })
    }

    pub async fn listen(&self) -> ! {
        loop {
            let (socket, addr) = self.tcp_listener.accept().await.unwrap();
            tracing::info!("Got socket (address {}), establishing connection...", addr);
            let connection = Connection::new(socket);
            connection.start_process().await;
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, FromPrimitive)]
pub enum ConnectionState {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Configuration = 3,
    Play = 4,
}

#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
    state: ConnectionState,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::zeroed(4096),
            state: ConnectionState::Handshaking,
        }
    }

    pub async fn start_process(mut self) -> JoinHandle<ConnectionResult<()>> {
        tokio::spawn(async move {
            loop {
                tracing::trace!("Waiting for packet...");
                match self.stream.read(&mut self.buffer).await {
                    // Remote has closed
                    Ok(0) => {
                        tracing::trace!("Remote has closed.");
                        return Ok(());
                    }
                    Ok(n) => {
                        tracing::trace!("Received {} bytes, attempting to read packet...", n);
                        if let Some(packet) = self.read_packet().await? {
                            todo!();
                        }
                    }
                    Err(err) => {
                        tracing::warn!("Unexpected socket error: {}.", err);
                        return Err(err.into());
                    }
                }
            }
        })
    }

    pub async fn read_packet(&mut self) -> ConnectionResult<Option<Packet>> {
        loop {
            if let Some(packet) = self.parse_packet()? {
                tracing::trace!("{:?}", packet);
                return Ok(Some(packet));
            }

            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(ConnectionError::ResetByPeer);
                }
            }
        }
    }

    pub fn parse_packet(&mut self) -> ConnectionResult<Option<Packet>> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Packet::check(&mut buf) {
            Ok(PacketCheckOutcome::Ok { len, packet_id }) => {
                let full_len = buf.position() as usize;
                let packet = Packet::decode(
                    len,
                    packet_id,
                    &mut buf.copy_to_bytes(len.try_into().unwrap()),
                )?;
                self.buffer.advance(full_len);
                Ok(Some(packet))
            }
            Ok(PacketCheckOutcome::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

type ConnectionResult<T> = Result<T, ConnectionError>;

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("packet decode error: {0}")]
    PacketDecode(#[from] PacketDecodeError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("connection reset by peer")]
    ResetByPeer,
}
