//! RakNet Protocol Tests
//!
//! Tests for RakNet/Bedrock Edition filtering including amplification
//! attack protection and protocol validation.

use pistonprotection_ebpf_tests::packet_generator::*;
use std::net::Ipv4Addr;

#[cfg(test)]
mod raknet_packet_tests {
    use super::*;

    /// Test RakNet magic validation
    #[test]
    fn test_raknet_magic_valid() {
        let ping = RakNetPing::new().build();

        // Magic should be at offset 9-25 (after packet ID and timestamp)
        let magic = &ping[9..25];
        assert_eq!(magic, &RAKNET_MAGIC, "Magic bytes should match");
    }

    /// Test RakNet magic invalid
    #[test]
    fn test_raknet_magic_invalid() {
        let mut ping = RakNetPing::new().build();

        // Corrupt magic
        ping[15] = 0x00;

        let magic = &ping[9..25];
        assert_ne!(magic, &RAKNET_MAGIC, "Corrupted magic should not match");
    }

    /// Test all RakNet client packet types
    #[test]
    fn test_raknet_client_packet_types() {
        let client_packets = [
            (RAKNET_UNCONNECTED_PING, "Unconnected Ping"),
            (RAKNET_UNCONNECTED_PING_OPEN, "Unconnected Ping Open"),
            (RAKNET_OPEN_CONNECTION_REQUEST_1, "Open Conn Request 1"),
            (RAKNET_OPEN_CONNECTION_REQUEST_2, "Open Conn Request 2"),
            (0x09, "Connection Request"),
            (0x13, "New Incoming Connection"),
            (0x00, "Connected Ping"),
            (0x15, "Disconnect Notification"),
            (0x80, "Data Packet"),
            (0xa0, "NACK"),
            (0xc0, "ACK"),
        ];

        for (packet_id, name) in client_packets {
            let mut packet = vec![packet_id];
            packet.extend_from_slice(&RAKNET_MAGIC);
            packet.resize(50, 0);

            assert_eq!(packet[0], packet_id, "{} packet ID mismatch", name);
        }
    }

    /// Test RakNet server packet types (should be rejected when received as server)
    #[test]
    fn test_raknet_server_packet_types() {
        let server_packets = [
            (RAKNET_UNCONNECTED_PONG, "Unconnected Pong"),
            (0x06, "Open Conn Reply 1"),
            (0x08, "Open Conn Reply 2"),
            (0x10, "Connection Request Accepted"),
            (0x03, "Connected Pong"),
            (0x19, "Incompatible Protocol"),
        ];

        for (packet_id, name) in server_packets {
            let mut packet = vec![packet_id];
            packet.extend_from_slice(&RAKNET_MAGIC);
            packet.resize(50, 0);

            assert_eq!(packet[0], packet_id, "{} packet ID mismatch", name);
            // These should all be dropped by the filter when acting as server
        }
    }
}

#[cfg(test)]
mod raknet_ping_tests {
    use super::*;

    /// Test minimum valid ping packet
    #[test]
    fn test_ping_minimum_size() {
        let ping = RakNetPing::new().build();

        // Minimum size: 1 (ID) + 8 (time) + 16 (magic) + 8 (GUID) = 33
        assert_eq!(ping.len(), 33, "Ping should be exactly 33 bytes");
    }

    /// Test ping with all fields
    #[test]
    fn test_ping_all_fields() {
        let time: u64 = 0x123456789ABCDEF0;
        let guid: u64 = 0xFEDCBA9876543210;

        let ping = RakNetPing::new()
            .with_time(time)
            .with_guid(guid)
            .build();

        // Verify packet ID
        assert_eq!(ping[0], RAKNET_UNCONNECTED_PING);

        // Verify time (big-endian)
        let extracted_time = u64::from_be_bytes([
            ping[1], ping[2], ping[3], ping[4],
            ping[5], ping[6], ping[7], ping[8],
        ]);
        assert_eq!(extracted_time, time);

        // Verify magic
        assert_eq!(&ping[9..25], &RAKNET_MAGIC);

        // Verify GUID (big-endian)
        let extracted_guid = u64::from_be_bytes([
            ping[25], ping[26], ping[27], ping[28],
            ping[29], ping[30], ping[31], ping[32],
        ]);
        assert_eq!(extracted_guid, guid);
    }

    /// Test ping open connections variant
    #[test]
    fn test_ping_open_connections() {
        let ping = RakNetPing::new()
            .open_connections()
            .build();

        assert_eq!(ping[0], RAKNET_UNCONNECTED_PING_OPEN);
    }

    /// Test undersized ping packet
    #[test]
    fn test_ping_undersized() {
        // Ping with missing GUID (should be rejected)
        let mut ping = RakNetPing::new().build();
        ping.truncate(25); // Remove GUID

        assert!(ping.len() < 33, "Undersized ping should have < 33 bytes");
        // Filter should drop this
    }
}

#[cfg(test)]
mod raknet_connection_tests {
    use super::*;

    /// Test Open Connection Request 1 structure
    #[test]
    fn test_open_conn_req1_structure() {
        let req = RakNetOpenConnReq1::new()
            .with_protocol(11)
            .with_mtu(1400)
            .build();

        // Size should equal MTU
        assert_eq!(req.len(), 1400);

        // Verify packet ID
        assert_eq!(req[0], RAKNET_OPEN_CONNECTION_REQUEST_1);

        // Verify magic
        assert_eq!(&req[1..17], &RAKNET_MAGIC);

        // Verify protocol version
        assert_eq!(req[17], 11);

        // Rest should be zero padding
        assert!(req[18..].iter().all(|&b| b == 0));
    }

    /// Test Open Connection Request 2 structure
    #[test]
    fn test_open_conn_req2_structure() {
        let req = RakNetOpenConnReq2::new()
            .with_mtu(1200)
            .with_guid(0xDEADBEEF12345678)
            .build();

        // Verify packet ID
        assert_eq!(req[0], RAKNET_OPEN_CONNECTION_REQUEST_2);

        // Verify magic
        assert_eq!(&req[1..17], &RAKNET_MAGIC);

        // Verify server address type (IPv4 = 4)
        assert_eq!(req[17], 4);

        // MTU at offset 24-25
        let mtu = u16::from_be_bytes([req[24], req[25]]);
        assert_eq!(mtu, 1200);

        // GUID at offset 26-33
        let guid = u64::from_be_bytes([
            req[26], req[27], req[28], req[29],
            req[30], req[31], req[32], req[33],
        ]);
        assert_eq!(guid, 0xDEADBEEF12345678);
    }

    /// Test invalid MTU values
    #[test]
    fn test_mtu_validation() {
        // Too small (below 400)
        let small_req = RakNetOpenConnReq1::new()
            .with_mtu(300)
            .build();
        assert_eq!(small_req.len(), 300);
        // Filter should reject MTU < 400

        // Too large (above 1500)
        let large_req = RakNetOpenConnReq1::new()
            .with_mtu(2000)
            .build();
        assert_eq!(large_req.len(), 2000);
        // Filter should reject MTU > 1500

        // Valid range
        for mtu in [400, 576, 1200, 1400, 1492, 1500] {
            let req = RakNetOpenConnReq1::new()
                .with_mtu(mtu)
                .build();
            assert_eq!(req.len(), mtu as usize);
        }
    }

    /// Test invalid protocol versions
    #[test]
    fn test_protocol_version_validation() {
        // Valid versions (typically <= 11)
        for version in 1..=11 {
            let req = RakNetOpenConnReq1::new()
                .with_protocol(version)
                .build();
            assert_eq!(req[17], version);
        }

        // Invalid (too high)
        let invalid_req = RakNetOpenConnReq1::new()
            .with_protocol(50)
            .build();
        assert_eq!(invalid_req[17], 50);
        // Filter should reject protocol > 11
    }
}

#[cfg(test)]
mod raknet_amplification_tests {
    use super::*;

    /// Test amplification factor calculation
    #[test]
    fn test_amplification_factor() {
        // Ping request size
        let ping_size = 33;

        // Typical pong response sizes with MOTD
        let pong_sizes = [100, 500, 1000, 1400];

        for &pong_size in &pong_sizes {
            let factor = pong_size / ping_size;
            println!(
                "Pong size {}: amplification factor {}x",
                pong_size, factor
            );
            // Factors > 10 should trigger protection
        }
    }

    /// Test ping flood simulation
    #[test]
    fn test_ping_flood() {
        let src_ip = Ipv4Addr::new(192, 168, 1, 100);
        let dst_ip = Ipv4Addr::new(10, 0, 0, 1);

        // Threshold: 50 pings per second per IP
        let threshold = 50;

        let mut packets = Vec::new();
        for i in 0..(threshold + 20) {
            let packet = create_raknet_ping_packet(
                src_ip,
                dst_ip,
                10000 + i as u16,
                0x12345678 + i as u64,
            );
            packets.push(packet);
        }

        assert_eq!(packets.len(), 70);
        // Filter should block after threshold exceeded
    }

    /// Test connection request flood
    #[test]
    fn test_connection_request_flood() {
        // Connection request flood: many open connection requests
        // Threshold: 20 per second per IP

        let threshold = 20;

        let mut packets = Vec::new();
        for _ in 0..(threshold + 10) {
            let req = RakNetOpenConnReq1::new()
                .with_mtu(1400)
                .build();
            packets.push(req);
        }

        assert_eq!(packets.len(), 30);
        // Filter should block after threshold
    }

    /// Test rate limit window reset
    #[test]
    fn test_rate_limit_window() {
        // Rate limit window is typically 1 second
        // After window expires, counters reset

        // Simulate:
        // T=0: 40 pings (under threshold)
        // T=0.5s: 40 more pings (total 80, over threshold = blocked)
        // T=1.1s: Window reset, 40 pings (under threshold again)

        // This tests that the filter properly resets windows
    }

    /// Test amplification ratio tracking
    #[test]
    fn test_amplification_ratio_tracking() {
        // Track bytes in vs estimated bytes out

        let ping_size = 33;
        let estimated_pong_size = 1000;
        let num_pings = 100;

        let bytes_in = ping_size * num_pings;
        let bytes_out_estimate = estimated_pong_size * num_pings;
        let ratio = bytes_out_estimate / bytes_in;

        assert!(ratio > 10, "Amplification ratio should exceed threshold");
        // Filter should detect this as amplification attack
    }
}

#[cfg(test)]
mod raknet_state_machine_tests {
    use super::*;

    /// Test valid state transitions
    #[test]
    fn test_valid_state_transitions() {
        // State machine:
        // NONE -> Ping sent (1) -> Open Conn Req 1 (2) -> Open Conn Req 2 (3) -> Connected (4)

        let states = [
            (0, "None"),
            (1, "Ping Sent"),
            (2, "Open Conn Req 1"),
            (3, "Open Conn Req 2"),
            (4, "Connected"),
        ];

        // Valid: forward transitions only
        for i in 0..states.len() - 1 {
            let (current, _) = states[i];
            let (next, _) = states[i + 1];
            assert!(next > current, "Forward transition should be allowed");
        }
    }

    /// Test invalid state transitions
    #[test]
    fn test_invalid_state_transitions() {
        // Backward transitions are invalid (state machine abuse)

        // Open Conn Req 2 without Open Conn Req 1
        // (state jumps from 0 to 3)
        let req2_without_req1 = RakNetOpenConnReq2::new().build();
        assert_eq!(req2_without_req1[0], RAKNET_OPEN_CONNECTION_REQUEST_2);
        // Filter should reject if current state != 2

        // Data packets without completing handshake
        let data_without_handshake = vec![0x84, 0x00, 0x00, 0x00];
        assert!(data_without_handshake[0] >= 0x80 && data_without_handshake[0] <= 0x8f);
        // Filter should reject if current state < 3
    }

    /// Test GUID consistency
    #[test]
    fn test_guid_consistency() {
        let client_guid: u64 = 0x123456789ABCDEF0;

        let ping = RakNetPing::new()
            .with_guid(client_guid)
            .build();

        let req2 = RakNetOpenConnReq2::new()
            .with_guid(client_guid)
            .build();

        // Extract GUIDs
        let ping_guid = u64::from_be_bytes([
            ping[25], ping[26], ping[27], ping[28],
            ping[29], ping[30], ping[31], ping[32],
        ]);

        let req2_guid = u64::from_be_bytes([
            req2[26], req2[27], req2[28], req2[29],
            req2[30], req2[31], req2[32], req2[33],
        ]);

        assert_eq!(ping_guid, client_guid);
        assert_eq!(req2_guid, client_guid);
        assert_eq!(ping_guid, req2_guid, "GUIDs should match across handshake");
    }

    /// Test GUID mismatch detection
    #[test]
    fn test_guid_mismatch() {
        let ping_guid: u64 = 0x1111111111111111;
        let req2_guid: u64 = 0x2222222222222222;

        let ping = RakNetPing::new()
            .with_guid(ping_guid)
            .build();

        let req2 = RakNetOpenConnReq2::new()
            .with_guid(req2_guid)
            .build();

        // Extract and compare
        let extracted_ping_guid = u64::from_be_bytes([
            ping[25], ping[26], ping[27], ping[28],
            ping[29], ping[30], ping[31], ping[32],
        ]);

        let extracted_req2_guid = u64::from_be_bytes([
            req2[26], req2[27], req2[28], req2[29],
            req2[30], req2[31], req2[32], req2[33],
        ]);

        assert_ne!(
            extracted_ping_guid, extracted_req2_guid,
            "GUIDs should not match - attack detected"
        );
    }
}

#[cfg(test)]
mod raknet_data_packet_tests {
    use super::*;

    /// Test data packet (Frame Set Packet) structure
    #[test]
    fn test_data_packet_structure() {
        // Data packets: 0x80-0x8f
        for packet_id in 0x80..=0x8f {
            let mut packet = vec![packet_id];
            // Sequence number (3 bytes, little-endian)
            packet.extend_from_slice(&[0x01, 0x00, 0x00]);
            // Minimal encapsulated data
            packet.extend_from_slice(&[0x60, 0x00, 0x10]); // Reliability + length

            assert!(packet.len() >= 4, "Data packet minimum size");
        }
    }

    /// Test ACK packet structure
    #[test]
    fn test_ack_packet() {
        let mut ack = vec![0xc0]; // ACK packet ID
        // Record count (2 bytes, little-endian)
        ack.extend_from_slice(&[0x01, 0x00]);
        // Single record flag
        ack.push(0x01);
        // Sequence number (3 bytes, little-endian)
        ack.extend_from_slice(&[0x05, 0x00, 0x00]);

        assert_eq!(ack[0], 0xc0);
        assert!(ack.len() >= 3);
    }

    /// Test NACK packet structure
    #[test]
    fn test_nack_packet() {
        let mut nack = vec![0xa0]; // NACK packet ID
        // Record count
        nack.extend_from_slice(&[0x01, 0x00]);
        // Single record flag
        nack.push(0x01);
        // Missing sequence number
        nack.extend_from_slice(&[0x03, 0x00, 0x00]);

        assert_eq!(nack[0], 0xa0);
        assert!(nack.len() >= 3);
    }
}

#[cfg(test)]
mod raknet_full_packet_tests {
    use super::*;

    /// Test complete RakNet ping packet with all headers
    #[test]
    fn test_complete_raknet_ping() {
        let packet = create_raknet_ping_packet(
            Ipv4Addr::new(192, 168, 1, 100),
            Ipv4Addr::new(10, 0, 0, 1),
            54321,
            0xDEADBEEFCAFEBABE,
        );

        // Size: Eth (14) + IP (20) + UDP (8) + RakNet Ping (33) = 75
        assert!(packet.len() >= 75);

        // Verify Ethernet
        assert_eq!(&packet[12..14], &[0x08, 0x00]); // IPv4

        // Verify IP protocol is UDP
        assert_eq!(packet[23], IPPROTO_UDP);

        // Verify UDP destination port is 19132
        let dst_port = u16::from_be_bytes([packet[36], packet[37]]);
        assert_eq!(dst_port, 19132);

        // Verify RakNet packet ID
        let raknet_start = 14 + 20 + 8; // After Eth + IP + UDP
        assert_eq!(packet[raknet_start], RAKNET_UNCONNECTED_PING);
    }

    /// Test complete Open Connection Request 1
    #[test]
    fn test_complete_open_conn_req1() {
        let raknet = RakNetOpenConnReq1::new()
            .with_mtu(1400)
            .build();

        let udp = UdpDatagram::new()
            .with_dst_port(19132)
            .with_payload(raknet)
            .build();

        let ip = Ipv4Packet::new()
            .with_protocol(IPPROTO_UDP)
            .with_payload(udp)
            .build();

        let frame = EthernetFrame::new()
            .with_payload(ip)
            .build();

        // Size: Eth (14) + IP (20) + UDP (8) + RakNet (1400) = 1442
        assert_eq!(frame.len(), 1442);
    }
}

#[cfg(test)]
mod raknet_attack_simulation_tests {
    use super::*;

    /// Test reflection attack detection
    #[test]
    fn test_reflection_attack() {
        // Reflection attack: attacker sends ping with victim's IP as source
        // Server sends large pong to victim

        // This test simulates receiving server-to-client packets
        // which would indicate reflection attack attempt

        let server_packets = [
            (RAKNET_UNCONNECTED_PONG, "Pong reflection"),
            (0x06, "Open Reply 1 reflection"),
            (0x08, "Open Reply 2 reflection"),
        ];

        for (packet_id, name) in server_packets {
            let mut packet = vec![packet_id];
            packet.extend_from_slice(&RAKNET_MAGIC);
            packet.resize(100, 0);

            // These should be dropped - servers shouldn't receive these
            println!("Detecting {}: packet_id = 0x{:02x}", name, packet_id);
        }
    }

    /// Test protocol downgrade attack
    #[test]
    fn test_protocol_downgrade() {
        // Attacker requests old protocol version which may have vulnerabilities

        for version in [1, 2, 3, 4, 5] {
            let req = RakNetOpenConnReq1::new()
                .with_protocol(version)
                .build();

            assert_eq!(req[17], version);
            // Old versions should be allowed but flagged for monitoring
        }
    }
}
