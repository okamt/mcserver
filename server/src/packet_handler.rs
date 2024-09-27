use std::{
    borrow::Cow,
    convert::Infallible,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use futures::future::BoxFuture;
use packet::{client::*, server::*, KnownPack, Packet};
use protocol::{identifier::Identifier, ConnectionState, EncodeError};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::connection::{Connection, PacketSendError, TARGET_PROTOCOL_VERSION};

pub trait PacketHandlerFn<P>:
    for<'a> FnMut(&'a P, &'a mut Connection) -> BoxFuture<'a, Result<(), PacketHandleError>> + Send
where
    P: Packet,
{
}

impl<T, P> PacketHandlerFn<P> for T
where
    T: for<'a> FnMut(&'a P, &'a mut Connection) -> BoxFuture<'a, Result<(), PacketHandleError>>
        + Send,
    P: Packet,
{
}

pub struct PacketHandler<P>
where
    P: Packet,
{
    handler: Box<dyn PacketHandlerFn<P>>,
}

impl<P> PacketHandler<P>
where
    P: Packet,
{
    pub fn new(handler: impl PacketHandlerFn<P> + 'static) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }
}

impl<P> Deref for PacketHandler<P>
where
    P: Packet,
{
    type Target = dyn PacketHandlerFn<P>;

    fn deref(&self) -> &Self::Target {
        &*self.handler
    }
}

impl<P> DerefMut for PacketHandler<P>
where
    P: Packet,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (*self.handler)
    }
}

pub struct PacketHandlerManager<'packet> {
    packet_handlers: Vec<PacketHandler<ServerPacket<'packet>>>,
}

impl<'packet> PacketHandlerManager<'packet> {
    /// Makes a new [`PacketHandlerManager`] with no handlers.
    pub fn empty() -> Self {
        Self {
            packet_handlers: Vec::new(),
        }
    }

    /// Adds a new [`PacketHandler`]. Handlers registered later will be executed first.
    pub fn push_handler(&mut self, handler: impl PacketHandlerFn<ServerPacket<'packet>> + 'static) {
        self.packet_handlers.push(PacketHandler::new(handler))
    }

    pub async fn handle_packet(
        &mut self,
        packet: impl Into<ServerPacket<'packet>>,
        connection: &mut Connection,
    ) -> Result<(), PacketHandleError> {
        let packet = packet.into();
        for handler in self.packet_handlers.iter_mut().rev() {
            match handler(&packet, connection).await {
                Ok(_) => {}
                err @ Err(_) => return err,
            }
        }

        Ok(())
    }
}

pub struct PacketHandlerManagerHandle<'a> {
    manager: Arc<Mutex<PacketHandlerManager<'a>>>,
}

impl<'a> PacketHandlerManagerHandle<'a> {
    pub fn new(manager: Arc<Mutex<PacketHandlerManager<'a>>>) -> Self {
        Self { manager }
    }

    pub async fn handle_packet(
        &mut self,
        packet: impl Into<ServerPacket<'a>>,
        connection: &mut Connection,
    ) -> Result<(), PacketHandleError> {
        let mut manager = self.manager.lock().await;
        manager.handle_packet(packet, connection).await
    }
}

#[derive(Error, Debug)]
pub enum PacketHandleError {
    #[error("incompatible protocol version: {0}")]
    IncompatibleProtocolVersion(i32),
    #[error(transparent)]
    PacketEncode(#[from] EncodeError<Infallible>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    PacketSend(#[from] PacketSendError),
    #[error("packet handling was cancelled")]
    Cancelled,
    #[error(transparent)]
    User(#[from] anyhow::Error),
}

pub async fn default_packet_handler(
    packet: &ServerPacket<'_>,
    connection: &mut Connection,
) -> Result<(), PacketHandleError> {
    match packet {
        ServerPacket::Handshaking(packet) => match packet {
            ServerHandshakingPacket::HandshakePacket(HandshakePacket {
                protocol_version,
                server_address,
                server_port,
                next_state,
            }) => {
                tracing::trace!(
                    "Connected through address {}:{}.",
                    server_address,
                    server_port
                );

                if *protocol_version != TARGET_PROTOCOL_VERSION {
                    tracing::trace!("Incompatible protocol version {}.", protocol_version);
                    return Err(PacketHandleError::IncompatibleProtocolVersion(
                        *protocol_version,
                    ));
                }

                tracing::trace!("Switching to state {:?}.", next_state);
                connection.state = *next_state;
                connection.can_request_status = true;
            }
        },
        ServerPacket::Status(packet) => match packet {
            ServerStatusPacket::StatusRequestPacket(StatusRequestPacket {}) => {
                if !connection.can_request_status {
                    tracing::trace!("Client is not currently allowed to request status, ignoring.");
                    return Ok(());
                }

                let status_response =
                    ClientStatusPacket::StatusResponsePacket(StatusResponsePacket {
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
                    });

                connection.send_packet(&status_response).await?;

                connection.can_request_status = false;
            }
            ServerStatusPacket::PingRequestPacket(PingRequestPacket { payload }) => {
                connection
                    .send_packet(&PongResponsePacket { payload: *payload })
                    .await?;
            }
        },
        ServerPacket::Login(packet) => match packet {
            ServerLoginPacket::LoginStartPacket(LoginStartPacket {
                player_username,
                player_uuid,
            }) => {
                // TODO: client auth, encryption, compression

                connection
                    .send_packet(&LoginSuccessPacket {
                        player_uuid: *player_uuid,
                        player_username: Cow::Borrowed(player_username),
                        properties: Vec::new().into(),
                        strict_error_handling: true,
                    })
                    .await?;
            }
            ServerLoginPacket::LoginAcknowledgedPacket(LoginAcknowledgedPacket {}) => {
                tracing::trace!("Login was acknowledged by the client.");

                connection.state = ConnectionState::Configuration;

                connection
                    .send_packet(&ClientboundKnownPacksPacket {
                        known_packs: (&[KnownPack {
                            identifier: Identifier::from_string("core").unwrap(),
                            version: "1.21".into(),
                        }])
                            .into(),
                    })
                    .await?;
            }
            _ => todo!(),
        },
        ServerPacket::Configuration(packet) => match packet {
            ServerConfigurationPacket::ServerboundPluginMessagePacket(
                ServerboundPluginMessagePacket {
                    channel_identifier,
                    data,
                },
            ) => {
                tracing::trace!(
                    "Received plugin message in channel {}: {:?}",
                    channel_identifier,
                    std::str::from_utf8(data)
                );

                // TODO
            }
            ServerConfigurationPacket::ClientInformationPacket(packet) => {
                connection.client_information = Some(packet.clone().into());
            }
            ServerConfigurationPacket::ServerboundKnownPacksPacket(_packet) => {
                // TODO
            }
        },
        ServerPacket::Play(_) => todo!(),
    }

    Ok(())
}
