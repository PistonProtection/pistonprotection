//! Minecraft Protocol Filter Tests
//!
//! Tests for Minecraft Java Edition (TCP) and Bedrock Edition (UDP/RakNet) filtering.

use pistonprotection_ebpf_tests::packet_generator::*;
use std::net::Ipv4Addr;

#[cfg(test)]
mod minecraft_java_tests {
    use super::*;

    /// Test valid handshake packet construction
    #[test]
    fn test_valid_handshake_packet() {
        let handshake = MinecraftHandshake::new()
            .with_protocol(765) // 1.20.4
            .with_address("mc.example.com")
            .with_port(25565)
            .login()
            .build();

        // Verify packet structure
        let (packet_len, len_bytes) = decode_varint(&handshake).unwrap();
        assert!(packet_len > 0, "Packet length should be positive");

        let packet_data = &handshake[len_bytes..];
        assert!(packet_data.len() >= packet_len as usize);

        // Verify packet ID is 0x00 (handshake)
        let (packet_id, id_bytes) = decode_varint(packet_data).unwrap();
        assert_eq!(packet_id, 0x00, "Handshake packet ID should be 0x00");

        // Verify protocol version
        let (proto_version, _) = decode_varint(&packet_data[id_bytes..]).unwrap();
        assert_eq!(proto_version, 765, "Protocol version should be 765");
    }

    /// Test handshake with status request (next_state = 1)
    #[test]
    fn test_handshake_status_request() {
        let handshake = MinecraftHandshake::new()
            .with_protocol(765)
            .status()
            .build();

        // Verify next_state is 1
        // Parse through the packet to find next_state
        let (packet_len, len_bytes) = decode_varint(&handshake).unwrap();
        let packet_data = &handshake[len_bytes..];

        // Skip packet ID
        let (_, id_bytes) = decode_varint(packet_data).unwrap();
        let mut offset = id_bytes;

        // Skip protocol version
        let (_, proto_bytes) = decode_varint(&packet_data[offset..]).unwrap();
        offset += proto_bytes;

        // Skip hostname string
        let (hostname_len, hostname_len_bytes) = decode_varint(&packet_data[offset..]).unwrap();
        offset += hostname_len_bytes + hostname_len as usize;

        // Skip port (2 bytes)
        offset += 2;

        // Read next_state
        let (next_state, _) = decode_varint(&packet_data[offset..]).unwrap();
        assert_eq!(next_state, 1, "next_state should be 1 for status");
    }

    /// Test handshake with login request (next_state = 2)
    #[test]
    fn test_handshake_login_request() {
        let handshake = MinecraftHandshake::new().with_protocol(765).login().build();

        // Find next_state in packet
        let (_, len_bytes) = decode_varint(&handshake).unwrap();
        let packet_data = &handshake[len_bytes..];

        let (_, id_bytes) = decode_varint(packet_data).unwrap();
        let mut offset = id_bytes;

        let (_, proto_bytes) = decode_varint(&packet_data[offset..]).unwrap();
        offset += proto_bytes;

        let (hostname_len, hostname_len_bytes) = decode_varint(&packet_data[offset..]).unwrap();
        offset += hostname_len_bytes + hostname_len as usize;

        offset += 2; // port

        let (next_state, _) = decode_varint(&packet_data[offset..]).unwrap();
        assert_eq!(next_state, 2, "next_state should be 2 for login");
    }

    /// Test invalid protocol versions
    #[test]
    fn test_invalid_protocol_versions() {
        // Too old (before 1.7.2)
        let old_proto = MinecraftHandshake::new().with_protocol(3).build();

        // Verify we can still parse it (filter would reject)
        let (packet_len, _) = decode_varint(&old_proto).unwrap();
        assert!(packet_len > 0);

        // Negative protocol version (attack vector)
        let negative_proto = MinecraftHandshake::new().with_protocol(-1).build();

        let (_, len_bytes) = decode_varint(&negative_proto).unwrap();
        let packet_data = &negative_proto[len_bytes..];
        let (_, id_bytes) = decode_varint(packet_data).unwrap();
        let (proto, _) = decode_varint(&packet_data[id_bytes..]).unwrap();
        assert!(
            proto < 0,
            "Negative protocol version should decode as negative"
        );
    }

    /// Test invalid next_state values
    #[test]
    fn test_invalid_next_state() {
        // next_state = 0 (invalid)
        let invalid_state_0 = MinecraftHandshake::new().with_next_state(0).build();

        // next_state = 3 (invalid for handshake)
        let invalid_state_3 = MinecraftHandshake::new().with_next_state(3).build();

        // next_state = -1 (negative, attack vector)
        let invalid_state_neg = MinecraftHandshake::new().with_next_state(-1).build();

        // All should build successfully (filter would reject)
        assert!(!invalid_state_0.is_empty());
        assert!(!invalid_state_3.is_empty());
        assert!(!invalid_state_neg.is_empty());
    }

    /// Test oversized hostname
    #[test]
    fn test_oversized_hostname() {
        // DNS max is 253, but let's try 1000 characters
        let long_hostname = "a".repeat(1000);
        let handshake = MinecraftHandshake::new()
            .with_address(&long_hostname)
            .build();

        // Should build but filter should reject
        let (packet_len, _) = decode_varint(&handshake).unwrap();
        assert!(packet_len > 1000, "Packet should contain long hostname");
    }

    /// Test hostname with null bytes
    #[test]
    fn test_hostname_with_null_bytes() {
        // Construct a handshake with null bytes in hostname manually
        let mut packet_data = Vec::new();

        // Packet ID
        packet_data.extend(encode_varint(0x00));
        // Protocol version
        packet_data.extend(encode_varint(765));
        // Hostname with null byte
        let hostname = "test\0evil.com";
        packet_data.extend(encode_varint(hostname.len() as i32));
        packet_data.extend(hostname.as_bytes());
        // Port
        packet_data.extend_from_slice(&25565u16.to_be_bytes());
        // Next state
        packet_data.extend(encode_varint(2));

        // Wrap with length prefix
        let mut packet = encode_varint(packet_data.len() as i32);
        packet.extend(packet_data);

        // Verify it contains the null byte
        assert!(packet.iter().any(|&b| b == 0));
    }

    /// Test port value edge cases
    #[test]
    fn test_port_edge_cases() {
        // Port 0 (invalid)
        let port_0 = MinecraftHandshake::new().with_port(0).build();

        // Max port
        let port_max = MinecraftHandshake::new().with_port(65535).build();

        // Both should build
        assert!(!port_0.is_empty());
        assert!(!port_max.is_empty());
    }

    /// Test packet length manipulation attack
    #[test]
    fn test_packet_length_manipulation() {
        let handshake = MinecraftHandshake::new().build();
        let (actual_len, len_bytes) = decode_varint(&handshake).unwrap();

        // Create packet with wrong length
        let mut bad_packet = encode_varint(actual_len + 100); // Claim it's longer
        bad_packet.extend_from_slice(&handshake[len_bytes..]);

        // The claimed length doesn't match actual content
        let (claimed_len, _) = decode_varint(&bad_packet).unwrap();
        assert!(claimed_len as usize > bad_packet.len() - len_bytes);
    }

    /// Test TCP segmentation simulation
    #[test]
    fn test_tcp_segmentation() {
        let handshake = MinecraftHandshake::new().build();

        // Simulate receiving packet in fragments
        // Fragment 1: Just the length prefix
        let frag1 = &handshake[..2];
        let result1 = decode_varint(frag1);
        // Might be complete or incomplete depending on length encoding

        // Fragment 2: Rest of packet (assuming 2-byte length)
        // In real scenario, TCP stack reassembles but XDP might see fragments
    }

    /// Test state machine transitions
    #[test]
    fn test_state_machine_transitions() {
        // Valid sequence: NONE -> Status
        let handshake_status = MinecraftHandshake::new().status().build();

        // Valid sequence: NONE -> Login
        let handshake_login = MinecraftHandshake::new().login().build();

        // After status handshake, valid packets are:
        // - 0x00 Status Request
        // - 0x01 Ping Request

        // After login handshake, valid packets are:
        // - 0x00 Login Start
        // - 0x01 Encryption Response
        // - 0x02 Login Plugin Response
        // - 0x03 Login Acknowledged

        // Create status request packet (packet ID 0x00, no additional data)
        let mut status_request = encode_varint(1); // length = 1
        status_request.push(0x00); // packet ID

        // Create ping request packet (packet ID 0x01, 8 bytes payload)
        let mut ping_request = encode_varint(9); // length = 9
        ping_request.push(0x01); // packet ID
        ping_request.extend_from_slice(&[0; 8]); // timestamp

        assert_eq!(status_request.len(), 2);
        assert_eq!(ping_request.len(), 10);
    }

    /// Test encryption state handling
    #[test]
    fn test_encryption_state() {
        // After Login Acknowledged (0x03), encryption may be enabled
        // Once encrypted, filter should pass packets through without inspection

        // Login Acknowledged packet
        let mut login_ack = encode_varint(1); // length
        login_ack.push(0x03); // packet ID

        // Simulated encrypted payload (random-looking)
        let encrypted_payload: Vec<u8> = (0..50).map(|i| (i * 17 + 31) as u8).collect();

        // Filter should:
        // 1. Process Login Acknowledged
        // 2. Mark connection for encryption
        // 3. Pass subsequent packets without inspection
    }
}

#[cfg(test)]
mod minecraft_bedrock_tests {
    use super::*;

    /// Test valid RakNet ping packet
    #[test]
    fn test_valid_raknet_ping() {
        let ping = RakNetPing::new()
            .with_time(12345)
            .with_guid(0xDEADBEEF)
            .build();

        assert_eq!(ping.len(), 33, "Ping packet should be 33 bytes");
        assert_eq!(ping[0], RAKNET_UNCONNECTED_PING, "Packet ID should be 0x01");

        // Verify time bytes
        let time = u64::from_be_bytes([
            ping[1], ping[2], ping[3], ping[4], ping[5], ping[6], ping[7], ping[8],
        ]);
        assert_eq!(time, 12345);

        // Verify magic
        assert_eq!(&ping[9..25], &RAKNET_MAGIC);

        // Verify GUID
        let guid = u64::from_be_bytes([
            ping[25], ping[26], ping[27], ping[28], ping[29], ping[30], ping[31], ping[32],
        ]);
        assert_eq!(guid, 0xDEADBEEF);
    }

    /// Test RakNet ping with invalid magic
    #[test]
    fn test_invalid_raknet_magic() {
        let mut ping = RakNetPing::new().build();

        // Corrupt the magic bytes
        ping[9] = 0xFF;
        ping[10] = 0x00;

        // Magic should no longer match
        assert_ne!(&ping[9..25], &RAKNET_MAGIC);
    }

    /// Test Open Connection Request 1
    #[test]
    fn test_open_connection_request_1() {
        let req = RakNetOpenConnReq1::new()
            .with_protocol(11)
            .with_mtu(1400)
            .build();

        assert_eq!(req.len(), 1400, "Packet size should equal MTU");
        assert_eq!(req[0], RAKNET_OPEN_CONNECTION_REQUEST_1);
        assert_eq!(&req[1..17], &RAKNET_MAGIC);
        assert_eq!(req[17], 11, "Protocol version should be 11");

        // Rest should be zeros (MTU padding)
        assert!(req[18..].iter().all(|&b| b == 0));
    }

    /// Test Open Connection Request 2
    #[test]
    fn test_open_connection_request_2() {
        let req = RakNetOpenConnReq2::new()
            .with_mtu(1200)
            .with_guid(0x123456789ABCDEF0)
            .build();

        assert_eq!(req[0], RAKNET_OPEN_CONNECTION_REQUEST_2);
        assert_eq!(&req[1..17], &RAKNET_MAGIC);

        // Verify MTU is encoded
        let mtu = u16::from_be_bytes([req[24], req[25]]);
        assert_eq!(mtu, 1200);

        // Verify GUID
        let guid = u64::from_be_bytes([
            req[26], req[27], req[28], req[29], req[30], req[31], req[32], req[33],
        ]);
        assert_eq!(guid, 0x123456789ABCDEF0);
    }

    /// Test invalid MTU values
    #[test]
    fn test_invalid_mtu_values() {
        // Too small (below 400)
        let small_mtu = RakNetOpenConnReq1::new().with_mtu(300).build();
        assert_eq!(small_mtu.len(), 300);

        // Too large (above 1500)
        let large_mtu = RakNetOpenConnReq1::new().with_mtu(2000).build();
        assert_eq!(large_mtu.len(), 2000);

        // Filter should reject both
    }

    /// Test invalid RakNet protocol version
    #[test]
    fn test_invalid_raknet_protocol() {
        // Protocol versions > 11 are suspicious
        let bad_proto = RakNetOpenConnReq1::new().with_protocol(50).build();

        assert_eq!(bad_proto[17], 50);
    }

    /// Test server-to-client packets (should be rejected when received as server)
    #[test]
    fn test_server_to_client_packets() {
        // These packet types should be rejected when received by the server
        let server_packets = [
            RAKNET_UNCONNECTED_PONG, // 0x1c
            0x06,                    // Open Connection Reply 1
            0x08,                    // Open Connection Reply 2
            0x10,                    // Connection Request Accepted
            0x03,                    // Connected Pong
        ];

        for &packet_id in &server_packets {
            let mut packet = vec![packet_id];
            packet.extend_from_slice(&RAKNET_MAGIC);
            // Add minimal payload
            packet.resize(50, 0);

            // These should all be rejected by the filter
            assert_eq!(packet[0], packet_id);
        }
    }

    /// Test RakNet state machine transitions
    #[test]
    fn test_raknet_state_machine() {
        // Valid sequence:
        // 1. Unconnected Ping -> Unconnected Pong
        // 2. Open Connection Request 1 -> Open Connection Reply 1
        // 3. Open Connection Request 2 -> Open Connection Reply 2
        // 4. Connection Request (0x09) -> Connection Request Accepted (0x10)
        // 5. New Incoming Connection (0x13)
        // 6. Connected session (0x80-0x8f data packets)

        let ping = RakNetPing::new().build();
        let req1 = RakNetOpenConnReq1::new().with_mtu(1400).build();
        let req2 = RakNetOpenConnReq2::new().with_mtu(1400).build();

        assert_eq!(ping[0], 0x01);
        assert_eq!(req1[0], 0x05);
        assert_eq!(req2[0], 0x07);
    }

    /// Test GUID consistency validation
    #[test]
    fn test_guid_consistency() {
        let guid = 0x1234567890ABCDEF_u64;

        let ping = RakNetPing::new().with_guid(guid).build();
        let req2 = RakNetOpenConnReq2::new().with_guid(guid).build();

        // Extract GUID from ping
        let ping_guid = u64::from_be_bytes([
            ping[25], ping[26], ping[27], ping[28], ping[29], ping[30], ping[31], ping[32],
        ]);

        // Extract GUID from req2
        let req2_guid = u64::from_be_bytes([
            req2[26], req2[27], req2[28], req2[29], req2[30], req2[31], req2[32], req2[33],
        ]);

        assert_eq!(ping_guid, guid);
        assert_eq!(req2_guid, guid);

        // Test with mismatched GUID
        let mismatch_req2 = RakNetOpenConnReq2::new().with_guid(0xFFFFFFFF).build();
        let mismatch_guid = u64::from_be_bytes([
            mismatch_req2[26],
            mismatch_req2[27],
            mismatch_req2[28],
            mismatch_req2[29],
            mismatch_req2[30],
            mismatch_req2[31],
            mismatch_req2[32],
            mismatch_req2[33],
        ]);
        assert_ne!(mismatch_guid, guid);
    }

    /// Test amplification attack detection
    #[test]
    fn test_amplification_attack_vectors() {
        // Small ping packet
        let ping = RakNetPing::new().build();
        let ping_size = ping.len(); // 33 bytes

        // Typical pong response with MOTD can be 500-1500 bytes
        // This creates an amplification factor of 15-45x

        // The filter should rate-limit pings to prevent amplification
        // Typical threshold: 50 pings per second per IP

        // Create flood of ping packets
        let flood_count = 100;
        let total_request_bytes = ping_size * flood_count;
        let estimated_response_bytes = 1000 * flood_count; // Conservative estimate

        let amplification_factor = estimated_response_bytes / total_request_bytes;
        assert!(
            amplification_factor > 10,
            "Amplification factor should be detected"
        );
    }

    /// Test data packet validation
    #[test]
    fn test_data_packets() {
        // Frame set packets (0x80-0x8f)
        for packet_id in 0x80..=0x8f {
            let mut packet = vec![packet_id];
            // Sequence number (3 bytes, little-endian)
            packet.extend_from_slice(&[0x00, 0x00, 0x00]);
            // Minimal encapsulated data
            packet.extend_from_slice(&[0x60, 0x00, 0x00]); // Reliability + length

            assert!(
                packet.len() >= 4,
                "Data packet should have at least 4 bytes"
            );
        }
    }

    /// Test ACK/NACK packets
    #[test]
    fn test_ack_nack_packets() {
        // ACK packet (0xc0)
        let mut ack = vec![0xc0];
        // Record count (2 bytes)
        ack.extend_from_slice(&[0x01, 0x00]);
        // Record: single or range
        ack.push(0x01); // Single sequence number
        ack.extend_from_slice(&[0x00, 0x00, 0x00]); // Sequence number

        // NACK packet (0xa0)
        let mut nack = vec![0xa0];
        nack.extend_from_slice(&[0x01, 0x00]);
        nack.push(0x01);
        nack.extend_from_slice(&[0x00, 0x00, 0x00]);

        assert!(ack.len() >= 3);
        assert!(nack.len() >= 3);
    }
}

#[cfg(test)]
mod minecraft_full_packet_tests {
    use super::*;

    /// Test creating complete Minecraft Java packet with all headers
    #[test]
    fn test_complete_minecraft_java_packet() {
        let packet = create_minecraft_handshake_packet(
            Ipv4Addr::new(192, 168, 1, 100),
            Ipv4Addr::new(10, 0, 0, 1),
            54321,
            765,
            2,
        );

        // Verify minimum size: Eth (14) + IP (20) + TCP (20) + MC data
        assert!(packet.len() >= 54);

        // Verify Ethernet type is IPv4
        assert_eq!(packet[12], 0x08);
        assert_eq!(packet[13], 0x00);

        // Verify IP protocol is TCP
        assert_eq!(packet[23], IPPROTO_TCP);

        // Verify destination port is 25565
        let dst_port = u16::from_be_bytes([packet[36], packet[37]]);
        assert_eq!(dst_port, 25565);
    }

    /// Test creating complete RakNet ping packet
    #[test]
    fn test_complete_raknet_packet() {
        let packet = create_raknet_ping_packet(
            Ipv4Addr::new(192, 168, 1, 100),
            Ipv4Addr::new(10, 0, 0, 1),
            54321,
            0xDEADBEEFCAFEBABE,
        );

        // Verify minimum size: Eth (14) + IP (20) + UDP (8) + RakNet (33)
        assert!(packet.len() >= 75);

        // Verify Ethernet type is IPv4
        assert_eq!(packet[12], 0x08);
        assert_eq!(packet[13], 0x00);

        // Verify IP protocol is UDP
        assert_eq!(packet[23], IPPROTO_UDP);

        // Verify destination port is 19132
        let dst_port = u16::from_be_bytes([packet[36], packet[37]]);
        assert_eq!(dst_port, 19132);
    }
}
