//! Minecraft offline fallback message generation.
//!
//! When the backend Minecraft server is unavailable, this module generates
//! proper Minecraft protocol responses to display a disconnect message
//! to connecting players instead of a connection timeout.

use serde::Serialize;
use tracing::debug;

/// Configuration for offline fallback messages.
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// Message to display when server is offline (supports Minecraft color codes)
    pub disconnect_message: String,
    /// MOTD to show in server list when offline
    pub motd: String,
    /// Protocol version to report (for version compatibility display)
    pub protocol_version: i32,
    /// Version name to display
    pub version_name: String,
    /// Maximum players to show in server list
    pub max_players: u32,
    /// Online players to show (usually 0 when offline)
    pub online_players: u32,
    /// Optional favicon (base64 encoded PNG, 64x64)
    pub favicon: Option<String>,
    /// Sample players to show in server list
    pub sample_players: Vec<SamplePlayer>,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            disconnect_message: "§cServer is currently offline.\n§7Please try again later."
                .to_string(),
            motd: "§c§lOFFLINE §8| §7Server maintenance in progress".to_string(),
            protocol_version: 767, // 1.21
            version_name: "Maintenance".to_string(),
            max_players: 0,
            online_players: 0,
            favicon: None,
            sample_players: vec![SamplePlayer {
                name: "§7Server is offline".to_string(),
                id: "00000000-0000-0000-0000-000000000000".to_string(),
            }],
        }
    }
}

/// Sample player entry for server list.
#[derive(Debug, Clone, Serialize)]
pub struct SamplePlayer {
    pub name: String,
    #[serde(rename = "id")]
    pub id: String,
}

/// Server list ping response (JSON format).
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub version: VersionInfo,
    pub players: PlayersInfo,
    pub description: Description,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(rename = "enforcesSecureChat", skip_serializing_if = "Option::is_none")]
    pub enforces_secure_chat: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Serialize)]
pub struct PlayersInfo {
    pub max: u32,
    pub online: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sample: Vec<SamplePlayer>,
}

#[derive(Debug, Serialize)]
pub struct Description {
    pub text: String,
}

/// Minecraft packet builder for fallback responses.
pub struct MinecraftPacketBuilder;

impl MinecraftPacketBuilder {
    /// Build a VarInt.
    pub fn write_varint(value: i32) -> Vec<u8> {
        let mut buf = Vec::with_capacity(5);
        let mut val = value as u32;

        loop {
            let mut byte = (val & 0x7f) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80;
            }
            buf.push(byte);
            if val == 0 {
                break;
            }
        }

        buf
    }

    /// Build a string (VarInt length + UTF-8 bytes).
    pub fn write_string(s: &str) -> Vec<u8> {
        let bytes = s.as_bytes();
        let mut buf = Self::write_varint(bytes.len() as i32);
        buf.extend_from_slice(bytes);
        buf
    }

    /// Build a complete packet with length prefix.
    pub fn build_packet(packet_id: i32, data: &[u8]) -> Vec<u8> {
        let id_bytes = Self::write_varint(packet_id);
        let total_len = id_bytes.len() + data.len();
        let mut packet = Self::write_varint(total_len as i32);
        packet.extend_from_slice(&id_bytes);
        packet.extend_from_slice(data);
        packet
    }

    /// Build a Disconnect packet (0x00 in login state, 0x1D in play state).
    pub fn build_disconnect_packet(message: &str, in_login_state: bool) -> Vec<u8> {
        // Minecraft expects a JSON Chat Component
        let json_message = serde_json::json!({
            "text": message
        });
        let json_str = serde_json::to_string(&json_message)
            .unwrap_or_else(|_| format!(r#"{{"text":"{}"}}"#, message));

        let data = Self::write_string(&json_str);

        // Packet ID: 0x00 for login disconnect, 0x1D for play disconnect (1.20+)
        let packet_id = if in_login_state { 0x00 } else { 0x1D };
        Self::build_packet(packet_id, &data)
    }

    /// Build a Status Response packet (0x00 in status state).
    pub fn build_status_response(config: &FallbackConfig) -> Vec<u8> {
        let response = StatusResponse {
            version: VersionInfo {
                name: config.version_name.clone(),
                protocol: config.protocol_version,
            },
            players: PlayersInfo {
                max: config.max_players,
                online: config.online_players,
                sample: config.sample_players.clone(),
            },
            description: Description {
                text: config.motd.clone(),
            },
            favicon: config.favicon.clone(),
            enforces_secure_chat: Some(false),
        };

        let json_str = serde_json::to_string(&response).unwrap_or_else(|_| {
            // Minimal fallback
            format!(
                r#"{{"version":{{"name":"{}","protocol":{}}},"players":{{"max":{},"online":{}}},"description":{{"text":"{}"}}}}"#,
                config.version_name,
                config.protocol_version,
                config.max_players,
                config.online_players,
                config.motd
            )
        });

        let data = Self::write_string(&json_str);
        Self::build_packet(0x00, &data)
    }

    /// Build a Ping Response packet (0x01 in status state).
    pub fn build_ping_response(payload: i64) -> Vec<u8> {
        let data = payload.to_be_bytes().to_vec();
        Self::build_packet(0x01, &data)
    }

    /// Build a Login Success packet (0x02 in login state) - for testing/simulation.
    #[allow(dead_code)]
    pub fn build_login_success(uuid: &str, username: &str) -> Vec<u8> {
        let mut data = Vec::new();

        // UUID (as bytes, 16 bytes for 1.16+)
        if let Ok(parsed_uuid) = uuid::Uuid::parse_str(uuid) {
            data.extend_from_slice(parsed_uuid.as_bytes());
        } else {
            // Fallback to nil UUID
            data.extend_from_slice(&[0u8; 16]);
        }

        // Username
        data.extend_from_slice(&Self::write_string(username));

        // Properties array (empty)
        data.extend_from_slice(&Self::write_varint(0));

        Self::build_packet(0x02, &data)
    }
}

/// Minecraft Bedrock/RakNet fallback packet builder.
pub struct BedrockPacketBuilder;

impl BedrockPacketBuilder {
    /// RakNet magic bytes.
    const MAGIC: [u8; 16] = [
        0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56,
        0x78,
    ];

    /// Build an Unconnected Pong response (0x1c).
    ///
    /// This is sent in response to Unconnected Ping to show server in server list.
    pub fn build_unconnected_pong(
        ping_time: i64,
        server_guid: i64,
        motd: &str,
        max_players: u32,
        online_players: u32,
        server_name: &str,
        protocol_version: u32,
    ) -> Vec<u8> {
        let mut packet = Vec::with_capacity(256);

        // Packet ID
        packet.push(0x1c);

        // Time (same as ping)
        packet.extend_from_slice(&ping_time.to_be_bytes());

        // Server GUID
        packet.extend_from_slice(&server_guid.to_be_bytes());

        // Magic
        packet.extend_from_slice(&Self::MAGIC);

        // MOTD string (format: Edition;MOTD;ProtocolVersion;Version;Players;MaxPlayers;ServerUniqueID;WorldName;Gamemode;NintendoLimited;Port;Port)
        let motd_parts = format!(
            "MCPE;{};{};1.21.0;{};{};{};{};;;19132;19133",
            motd.replace(';', " "), // Escape semicolons
            protocol_version,
            online_players,
            max_players,
            server_guid,
            server_name.replace(';', " ")
        );

        // String length (2 bytes, big-endian)
        let motd_bytes = motd_parts.as_bytes();
        packet.extend_from_slice(&(motd_bytes.len() as u16).to_be_bytes());
        packet.extend_from_slice(motd_bytes);

        packet
    }

    /// Build an Incompatible Protocol Version response (0x19).
    pub fn build_incompatible_protocol(server_protocol: u8, server_guid: i64) -> Vec<u8> {
        let mut packet = Vec::with_capacity(26);

        // Packet ID
        packet.push(0x19);

        // Server protocol version
        packet.push(server_protocol);

        // Magic
        packet.extend_from_slice(&Self::MAGIC);

        // Server GUID
        packet.extend_from_slice(&server_guid.to_be_bytes());

        packet
    }
}

/// Generate a fallback response based on the incoming packet.
pub fn generate_fallback_response(
    packet_data: &[u8],
    config: &FallbackConfig,
    state: MinecraftState,
) -> Option<Vec<u8>> {
    match state {
        MinecraftState::Status => {
            // Status state requires parsing the incoming packet
            if packet_data.is_empty() {
                return None;
            }
            generate_status_fallback(packet_data, config)
        }
        MinecraftState::Login => {
            // Login state - always send disconnect (don't need packet data)
            Some(MinecraftPacketBuilder::build_disconnect_packet(
                &config.disconnect_message,
                true,
            ))
        }
        MinecraftState::Play => {
            // Play state - always send disconnect (don't need packet data)
            Some(MinecraftPacketBuilder::build_disconnect_packet(
                &config.disconnect_message,
                false,
            ))
        }
        MinecraftState::Handshake | MinecraftState::Configuration => None,
    }
}

/// Generate status state fallback response.
fn generate_status_fallback(packet_data: &[u8], config: &FallbackConfig) -> Option<Vec<u8>> {
    // Parse packet to determine if it's status request (0x00) or ping (0x01)
    let (_, len_size) = read_varint(packet_data)?;
    let packet_id_byte = packet_data.get(len_size)?;

    match *packet_id_byte {
        0x00 => {
            // Status Request - respond with Status Response
            debug!("Generating fallback status response");
            Some(MinecraftPacketBuilder::build_status_response(config))
        }
        0x01 => {
            // Ping - respond with Pong
            if packet_data.len() >= len_size + 1 + 8 {
                let payload_start = len_size + 1;
                let payload = i64::from_be_bytes(
                    packet_data[payload_start..payload_start + 8]
                        .try_into()
                        .ok()?,
                );
                debug!("Generating fallback ping response");
                Some(MinecraftPacketBuilder::build_ping_response(payload))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Minecraft connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MinecraftState {
    #[default]
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

impl MinecraftState {
    /// Create from next_state value in handshake.
    pub fn from_next_state(next_state: u8) -> Self {
        match next_state {
            1 => Self::Status,
            2 => Self::Login,
            3 => Self::Configuration, // Transfer intent (1.20.5+)
            _ => Self::Handshake,
        }
    }
}

/// Read a VarInt from the buffer.
fn read_varint(buf: &[u8]) -> Option<(i32, usize)> {
    let mut value: i32 = 0;
    let mut position = 0;

    for (i, &byte) in buf.iter().take(5).enumerate() {
        if i == 4 && (byte & 0xf0) != 0 {
            return None;
        }

        value |= ((byte & 0x7f) as i32) << position;

        if byte & 0x80 == 0 {
            return Some((value, i + 1));
        }

        position += 7;

        if i == 4 {
            return None;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_varint() {
        assert_eq!(MinecraftPacketBuilder::write_varint(0), vec![0x00]);
        assert_eq!(MinecraftPacketBuilder::write_varint(1), vec![0x01]);
        assert_eq!(MinecraftPacketBuilder::write_varint(127), vec![0x7f]);
        assert_eq!(MinecraftPacketBuilder::write_varint(128), vec![0x80, 0x01]);
        assert_eq!(MinecraftPacketBuilder::write_varint(255), vec![0xff, 0x01]);
        assert_eq!(
            MinecraftPacketBuilder::write_varint(25565),
            vec![0xdd, 0xc7, 0x01]
        );
    }

    #[test]
    fn test_write_string() {
        let result = MinecraftPacketBuilder::write_string("hello");
        assert_eq!(result, vec![0x05, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_build_disconnect_packet() {
        let packet = MinecraftPacketBuilder::build_disconnect_packet("Test", true);
        assert!(!packet.is_empty());
        // Should start with length VarInt
        assert!(packet[0] > 0);
    }

    #[test]
    fn test_build_status_response() {
        let config = FallbackConfig::default();
        let packet = MinecraftPacketBuilder::build_status_response(&config);
        assert!(!packet.is_empty());

        // Verify it's valid JSON inside
        let (len, len_size) = read_varint(&packet).unwrap();
        assert!(len > 0);

        // Skip length and packet ID to get JSON
        let json_start = len_size + 1; // +1 for packet ID
        if packet.len() > json_start {
            let (json_len, json_len_size) = read_varint(&packet[json_start..]).unwrap();
            let json_bytes =
                &packet[json_start + json_len_size..json_start + json_len_size + json_len as usize];
            let json: serde_json::Value = serde_json::from_slice(json_bytes).unwrap();
            assert!(json.get("version").is_some());
            assert!(json.get("players").is_some());
            assert!(json.get("description").is_some());
        }
    }

    #[test]
    fn test_build_ping_response() {
        let packet = MinecraftPacketBuilder::build_ping_response(12345);
        // Length (1) + Packet ID (1) + Payload (8) = 10 bytes minimum
        assert!(packet.len() >= 10);
    }

    #[test]
    fn test_bedrock_pong() {
        let packet = BedrockPacketBuilder::build_unconnected_pong(
            12345,
            67890,
            "Test Server",
            100,
            50,
            "World",
            685,
        );

        assert_eq!(packet[0], 0x1c);
        // Check magic is present
        assert_eq!(&packet[17..33], &BedrockPacketBuilder::MAGIC);
    }

    #[test]
    fn test_bedrock_incompatible_protocol() {
        let packet = BedrockPacketBuilder::build_incompatible_protocol(11, 12345);
        assert_eq!(packet[0], 0x19);
        assert_eq!(packet[1], 11);
        assert_eq!(&packet[2..18], &BedrockPacketBuilder::MAGIC);
    }

    #[test]
    fn test_state_from_next_state() {
        assert_eq!(MinecraftState::from_next_state(1), MinecraftState::Status);
        assert_eq!(MinecraftState::from_next_state(2), MinecraftState::Login);
        assert_eq!(
            MinecraftState::from_next_state(3),
            MinecraftState::Configuration
        );
        assert_eq!(
            MinecraftState::from_next_state(0),
            MinecraftState::Handshake
        );
    }

    #[test]
    fn test_generate_fallback_disconnect() {
        let config = FallbackConfig::default();

        // Login state should always generate disconnect even with empty data
        let response = generate_fallback_response(&[], &config, MinecraftState::Login);
        assert!(response.is_some());
        let packet = response.unwrap();
        assert!(!packet.is_empty());

        // Play state should also generate disconnect
        let response = generate_fallback_response(&[], &config, MinecraftState::Play);
        assert!(response.is_some());

        // Status state with empty data should return None (needs packet to parse)
        let response = generate_fallback_response(&[], &config, MinecraftState::Status);
        assert!(response.is_none());

        // Handshake state should return None (can't fallback during handshake)
        let response = generate_fallback_response(&[], &config, MinecraftState::Handshake);
        assert!(response.is_none());
    }
}
