use std::convert::Infallible;
use std::io::Cursor;
use std::sync::Arc;

use bytes::{Buf, BytesMut};
use futures::FutureExt;
use packet::Packet;
use packet::PacketDecodeError;
use packet::PacketDirection;
use packet::{server::ServerPacket, PacketCheckOutcome, PacketDecodeContext};
use protocol::buf;
use protocol::DecodeError;
use protocol::EncodeError;
use protocol::{ClientInformation, ConnectionState, Decodable};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::JoinHandle,
};

use crate::packet_handler::default_packet_handler;
use crate::packet_handler::PacketHandlerManagerHandle;
use crate::packet_handler::{PacketHandleError, PacketHandlerManager};

pub const TARGET_PROTOCOL_VERSION: i32 = 767;

pub struct ConnectionManager {
    tcp_listener: TcpListener,
    packet_handler_manager: Arc<Mutex<PacketHandlerManager<'static>>>,
}

impl ConnectionManager {
    pub async fn new<A>(address: A) -> std::io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        let mut packet_handler_manager = PacketHandlerManager::empty();

        packet_handler_manager.push_handler(|packet, connection| {
            async move { default_packet_handler(packet, connection).await }.boxed()
        });

        Ok(Self {
            tcp_listener: TcpListener::bind(address).await?,
            packet_handler_manager: Arc::new(Mutex::new(packet_handler_manager)),
        })
    }

    pub async fn listen(&self) -> ! {
        loop {
            let (socket, addr) = self.tcp_listener.accept().await.unwrap();
            tracing::info!("Got socket (address {}), establishing connection...", addr);
            let connection = Connection::new(socket);
            connection
                .start_process(PacketHandlerManagerHandle::new(Arc::clone(
                    &self.packet_handler_manager,
                )))
                .await;
        }
    }
}

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
    pub(crate) state: ConnectionState,
    pub(crate) can_request_status: bool,
    pub(crate) client_information: Option<ClientInformation>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::zeroed(4096),
            state: ConnectionState::Handshaking,
            can_request_status: false,
            client_information: None,
        }
    }

    pub async fn start_process(
        mut self,
        mut packet_handler_manager_handle: PacketHandlerManagerHandle<'static>,
    ) -> JoinHandle<ConnectionResult<()>> {
        tokio::spawn(async move {
            loop {
                tracing::trace!("Waiting for packet...");
                match self.stream.read(&mut self.buffer).await {
                    Ok(0) => {
                        tracing::trace!("Remote has closed.");
                        return Ok(());
                    }
                    Ok(n) => {
                        tracing::trace!("Received {} bytes, attempting to read packet...", n);
                        if let Some(packet) = self.read_packet().await? {
                            packet_handler_manager_handle
                                .handle_packet(packet, &mut self)
                                .await?;
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

    pub async fn read_packet(&mut self) -> ConnectionResult<Option<ServerPacket<'static>>> {
        loop {
            if let Some(packet) = self.parse_packet()? {
                tracing::trace!("Got packet {:?}.", packet);
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

    pub fn parse_packet(&mut self) -> ConnectionResult<Option<ServerPacket<'static>>> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match packet::check_packet(&mut buf) {
            Ok(PacketCheckOutcome::Ok { len, packet_id }) => {
                let full_len = buf.position() as usize;
                let packet = ServerPacket::decode(
                    &mut buf.copy_to_bytes(len.try_into().unwrap()),
                    PacketDecodeContext {
                        connection_state: self.state,
                        packet_id,
                        direction: PacketDirection::Server,
                    },
                )?;
                self.buffer.advance(full_len);
                Ok(Some(packet))
            }
            Ok(PacketCheckOutcome::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn send_packet<P: Packet + std::fmt::Debug>(
        &mut self,
        packet: &P,
    ) -> SendPacketResult<()> {
        let mut encoded_packet = BytesMut::with_capacity(4096);
        packet.encode(&mut encoded_packet, ())?;
        let mut packet_id_buf = BytesMut::with_capacity(5);
        buf::put_varint(&mut packet_id_buf, packet.get_id());
        let mut len_buf = BytesMut::with_capacity(5);
        buf::put_varint(
            &mut len_buf,
            (encoded_packet.len() + packet_id_buf.len())
                .try_into()
                .unwrap(),
        );

        tracing::trace!("Sending packet {:?}...", packet);

        self.stream.write_all_buf(&mut len_buf).await?;
        self.stream.write_all_buf(&mut packet_id_buf).await?;
        self.stream.write_all(&encoded_packet).await?;

        Ok(())
    }
}

pub type ConnectionResult<T> = Result<T, ConnectionError>;

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("packet decode error: {0}")]
    PacketDecode(#[from] DecodeError<PacketDecodeError>),
    #[error("error while handling packet: {0}")]
    PacketHandle(#[from] PacketHandleError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("connection reset by peer")]
    ResetByPeer,
}

pub type SendPacketResult<T> = Result<T, PacketSendError>;

#[derive(Error, Debug)]
pub enum PacketSendError {
    #[error(transparent)]
    PacketEncode(#[from] EncodeError<Infallible>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
