use anyhow::Result;
use bevy::prelude::*;
use bevy_renet::renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig};
use clap::Args;
use std::net::{SocketAddr, UdpSocket};
use std::time::SystemTime;

use super::{Channel, DEFAULT_PORT, MAX_CLIENTS, PROTOCOL_ID};

pub(super) struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerSettings::default());
    }
}

#[derive(Args, Clone, Debug, PartialEq)]
pub(crate) struct ServerSettings {
    /// Server name that will be visible to other players.
    #[clap(short, long, default_value_t = ServerSettings::default().server_name)]
    pub(crate) server_name: String,

    /// IP address to bind.
    #[clap(short, long, default_value_t = ServerSettings::default().ip)]
    pub(crate) ip: String,

    /// Port to use.
    #[clap(short, long, default_value_t = ServerSettings::default().port)]
    pub(crate) port: u16,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            server_name: "My game".to_string(),
            ip: "127.0.0.1".to_string(),
            port: DEFAULT_PORT,
        }
    }
}

impl ServerSettings {
    pub(crate) fn create_server(&self) -> Result<RenetServer> {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let server_addr = SocketAddr::new(self.ip.parse()?, self.port);
        let socket = UdpSocket::bind(server_addr)?;
        let server_config = ServerConfig::new(
            MAX_CLIENTS,
            PROTOCOL_ID,
            socket.local_addr()?,
            ServerAuthentication::Unsecure,
        );
        let connection_config = RenetConnectionConfig {
            send_channels_config: Channel::config(),
            receive_channels_config: Channel::config(),
            ..Default::default()
        };

        RenetServer::new(current_time, server_config, connection_config, socket).map_err(From::from)
    }
}