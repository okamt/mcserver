use std::{
    borrow::{Borrow, Cow},
    io::Cursor,
};

use bytes::{Buf, BytesMut};
use num_derive::{FromPrimitive, ToPrimitive};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::packet::{
    check_packet,
    client::{
        ClientLoginPacket, ClientPacket, ClientStatusPacket, StatusResponse,
        StatusResponseDescription, StatusResponsePlayers, StatusResponsePlayersSample,
        StatusResponseVersion,
    },
    server::{ServerHandshakingPacket, ServerLoginPacket, ServerPacket, ServerStatusPacket},
    BufMutExt, Packet, PacketCheckOutcome, PacketDecodeError, PacketDecoder, PacketEncodeError,
};

pub const TARGET_PROTOCOL_VERSION: i32 = 767;

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

#[derive(Debug, Eq, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(i32)]
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
    can_request_status: bool,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::zeroed(4096),
            state: ConnectionState::Handshaking,
            can_request_status: false,
        }
    }

    pub async fn start_process(mut self) -> JoinHandle<ConnectionResult<()>> {
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
                            self.process_packet(packet).await?;
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

    pub async fn read_packet(&mut self) -> ConnectionResult<Option<ServerPacket>> {
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

    pub fn parse_packet(&mut self) -> ConnectionResult<Option<ServerPacket>> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match check_packet(&mut buf) {
            Ok(PacketCheckOutcome::Ok { len, packet_id }) => {
                let full_len = buf.position() as usize;
                let packet = ServerPacket::decode(
                    self.state,
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

    pub async fn process_packet(&mut self, packet: ServerPacket) -> ProcessPacketResult<()> {
        match packet {
            ServerPacket::Handshaking(packet) => match packet {
                ServerHandshakingPacket::Handshake {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                } => {
                    if protocol_version != TARGET_PROTOCOL_VERSION {
                        tracing::trace!("Incompatible protocol version {}.", protocol_version);
                        return Err(ProcessPacketError::IncompatibleProtocolVersion(
                            protocol_version,
                        ));
                    }

                    tracing::trace!("Switching to state {:?}.", next_state);
                    self.state = next_state;
                    self.can_request_status = true;
                }
            },
            ServerPacket::Status(packet) => match packet {
                ServerStatusPacket::StatusRequest => {
                    if !self.can_request_status {
                        tracing::trace!(
                            "Client is not currently allowed to request status, ignoring."
                        );
                        return Ok(());
                    }

                    let status_response = ClientStatusPacket::StatusResponse {
                        response: StatusResponse {
                            version: StatusResponseVersion {
                                name: "1.21.1".into(),
                                protocol: 767,
                            },
                            players: StatusResponsePlayers {
                                max: 10000000,
                                online: 1,
                                sample: vec![StatusResponsePlayersSample {
                                    name: "hi".into(),
                                    id: Uuid::new_v4(),
                                }],
                            },
                            description: StatusResponseDescription {
                                text: "Blazing fast server".into(),
                            },
                            favicon: "".into(),
                            enforces_secure_chat: false,
                        },
                    };

                    self.send_packet(&status_response.into()).await?;

                    self.can_request_status = false;
                }
                ServerStatusPacket::PingRequest { payload } => {
                    self.send_packet(&ClientStatusPacket::PongResponse { payload }.into())
                        .await?;
                }
            },
            ServerPacket::Login(packet) => match packet {
                ServerLoginPacket::LoginStart {
                    player_username,
                    player_uuid,
                } => {
                    // TODO: client auth, encryption, compression

                    self.send_packet(
                        &ClientLoginPacket::LoginSuccess {
                            player_uuid: player_uuid,
                            player_username: player_username,
                            properties: Vec::new(),
                            strict_error_handling: true,
                        }
                        .into(),
                    )
                    .await?;
                }
                ServerLoginPacket::LoginAcknowledged => {
                    tracing::trace!("Login was acknowledged by the client.");
                    self.state = ConnectionState::Configuration;
                }
            },
            ServerPacket::Configuration(_) => todo!(),
            ServerPacket::Play(_) => todo!(),
        }

        Ok(())
    }

    pub async fn send_packet(&mut self, packet: &ClientPacket) -> SendPacketResult<()> {
        let encoded_packet = packet.encode()?;
        let mut packet_id_buf = BytesMut::with_capacity(5);
        packet_id_buf.put_varint(packet.get_id());
        let mut len_buf = BytesMut::with_capacity(5);
        len_buf.put_varint(
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
    PacketDecode(#[from] PacketDecodeError),
    #[error("error while processing packet: {0}")]
    ProcessPacket(#[from] ProcessPacketError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("connection reset by peer")]
    ResetByPeer,
}

pub type ProcessPacketResult<T> = Result<T, ProcessPacketError>;

#[derive(Error, Debug)]
pub enum ProcessPacketError {
    #[error("incompatible protocol version: {0}")]
    IncompatibleProtocolVersion(i32),
    #[error(transparent)]
    PacketEncode(#[from] PacketEncodeError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    SendPacket(#[from] SendPacketError),
}

pub type SendPacketResult<T> = Result<T, SendPacketError>;

#[derive(Error, Debug)]
pub enum SendPacketError {
    #[error(transparent)]
    PacketEncode(#[from] PacketEncodeError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
