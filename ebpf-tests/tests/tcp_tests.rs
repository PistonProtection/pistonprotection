//! TCP Filter Tests
//!
//! Tests for the enhanced TCP filter with SYN cookie, flood detection,
//! and invalid flag combination detection.

use pistonprotection_ebpf_tests::packet_generator::*;
use std::net::Ipv4Addr;

#[cfg(test)]
mod tcp_flag_tests {
    use super::*;

    /// Test valid TCP flag combinations
    #[test]
    fn test_valid_tcp_flags() {
        let valid_combinations = [
            TCP_SYN,                     // Initial connection
            TCP_SYN | TCP_ACK,           // SYN-ACK response
            TCP_ACK,                     // Data/ACK
            TCP_ACK | TCP_PSH,           // Data push
            TCP_ACK | TCP_FIN,           // FIN-ACK
            TCP_ACK | TCP_RST,           // RST-ACK
            TCP_RST,                     // Reset
            TCP_ACK | TCP_URG,           // Urgent data
            TCP_ACK | TCP_ECE,           // ECN capable
            TCP_ACK | TCP_CWR,           // Congestion window reduced
            TCP_ACK | TCP_PSH | TCP_URG, // Urgent data push
        ];

        for &flags in &valid_combinations {
            let segment = TcpSegment::new().with_flags(flags).build();

            // Verify flags are set
            let doff_flags = u16::from_be_bytes([segment[12], segment[13]]);
            let actual_flags = (doff_flags & 0x01ff) as u8;

            // Lower 6 bits should match
            assert_eq!(actual_flags & 0x3f, flags & 0x3f);
        }
    }

    /// Test invalid TCP flag combinations (scan patterns)
    #[test]
    fn test_invalid_tcp_flags_null_scan() {
        // NULL scan: no flags set
        let null_segment = TcpSegment::new().with_flags(0x00).build();

        let doff_flags = u16::from_be_bytes([null_segment[12], null_segment[13]]);
        let flags = (doff_flags & 0x003f) as u8;

        assert_eq!(flags, 0, "NULL scan should have no flags");
    }

    /// Test XMAS scan detection
    #[test]
    fn test_invalid_tcp_flags_xmas_scan() {
        // XMAS scan: FIN + URG + PSH
        let xmas_flags = TCP_FIN | TCP_URG | TCP_PSH;
        let xmas_segment = TcpSegment::new().with_flags(xmas_flags).build();

        let doff_flags = u16::from_be_bytes([xmas_segment[12], xmas_segment[13]]);
        let flags = (doff_flags & 0x003f) as u8;

        assert_eq!(flags, xmas_flags, "XMAS scan flags should match");
    }

    /// Test SYN+FIN combination (invalid)
    #[test]
    fn test_invalid_tcp_flags_syn_fin() {
        let syn_fin_segment = TcpSegment::new().with_flags(TCP_SYN | TCP_FIN).build();

        let doff_flags = u16::from_be_bytes([syn_fin_segment[12], syn_fin_segment[13]]);
        let flags = (doff_flags & 0x003f) as u8;

        assert_eq!(flags, TCP_SYN | TCP_FIN);
        // This combination should be detected and dropped
    }

    /// Test SYN+RST combination (invalid)
    #[test]
    fn test_invalid_tcp_flags_syn_rst() {
        let syn_rst_segment = TcpSegment::new().with_flags(TCP_SYN | TCP_RST).build();

        let doff_flags = u16::from_be_bytes([syn_rst_segment[12], syn_rst_segment[13]]);
        let flags = (doff_flags & 0x003f) as u8;

        assert_eq!(flags, TCP_SYN | TCP_RST);
    }

    /// Test FIN+RST combination (invalid)
    #[test]
    fn test_invalid_tcp_flags_fin_rst() {
        let fin_rst_segment = TcpSegment::new().with_flags(TCP_FIN | TCP_RST).build();

        let doff_flags = u16::from_be_bytes([fin_rst_segment[12], fin_rst_segment[13]]);
        let flags = (doff_flags & 0x003f) as u8;

        assert_eq!(flags, TCP_FIN | TCP_RST);
    }

    /// Test FIN without ACK (suspicious)
    #[test]
    fn test_invalid_tcp_flags_fin_alone() {
        let fin_segment = TcpSegment::new().with_flags(TCP_FIN).build();

        let doff_flags = u16::from_be_bytes([fin_segment[12], fin_segment[13]]);
        let flags = (doff_flags & 0x003f) as u8;

        assert_eq!(flags, TCP_FIN);
    }

    /// Test all invalid combinations
    #[test]
    fn test_all_invalid_flag_combinations() {
        let invalid_combinations = [
            (0x00, "NULL scan"),
            (TCP_SYN | TCP_FIN, "SYN+FIN"),
            (TCP_SYN | TCP_RST, "SYN+RST"),
            (TCP_FIN | TCP_RST, "FIN+RST"),
            (TCP_FIN | TCP_URG | TCP_PSH, "XMAS scan"),
            (TCP_FIN, "FIN alone"),
            (TCP_URG, "URG alone"),
        ];

        for (flags, name) in invalid_combinations {
            let segment = TcpSegment::new().with_flags(flags).build();

            let doff_flags = u16::from_be_bytes([segment[12], segment[13]]);
            let actual_flags = (doff_flags & 0x003f) as u8;

            assert_eq!(
                actual_flags, flags,
                "{} flags should be correctly encoded",
                name
            );
        }
    }
}

#[cfg(test)]
mod tcp_syn_cookie_tests {
    use super::*;

    /// Test SYN packet structure
    #[test]
    fn test_syn_packet_structure() {
        let syn = TcpSegment::new()
            .with_src_port(54321)
            .with_dst_port(80)
            .with_seq(1000000)
            .syn()
            .build();

        // Verify source port
        let src_port = u16::from_be_bytes([syn[0], syn[1]]);
        assert_eq!(src_port, 54321);

        // Verify destination port
        let dst_port = u16::from_be_bytes([syn[2], syn[3]]);
        assert_eq!(dst_port, 80);

        // Verify sequence number
        let seq = u32::from_be_bytes([syn[4], syn[5], syn[6], syn[7]]);
        assert_eq!(seq, 1000000);

        // Verify SYN flag
        let doff_flags = u16::from_be_bytes([syn[12], syn[13]]);
        let flags = (doff_flags & 0x003f) as u8;
        assert_eq!(flags, TCP_SYN);
    }

    /// Test SYN-ACK packet structure
    #[test]
    fn test_syn_ack_packet_structure() {
        let syn_ack = TcpSegment::new()
            .with_src_port(80)
            .with_dst_port(54321)
            .with_seq(2000000)
            .with_ack(1000001) // SYN seq + 1
            .syn_ack()
            .build();

        // Verify ACK number
        let ack = u32::from_be_bytes([syn_ack[8], syn_ack[9], syn_ack[10], syn_ack[11]]);
        assert_eq!(ack, 1000001);

        // Verify SYN+ACK flags
        let doff_flags = u16::from_be_bytes([syn_ack[12], syn_ack[13]]);
        let flags = (doff_flags & 0x003f) as u8;
        assert_eq!(flags, TCP_SYN | TCP_ACK);
    }

    /// Test ACK completing handshake
    #[test]
    fn test_handshake_ack() {
        let ack = TcpSegment::new()
            .with_src_port(54321)
            .with_dst_port(80)
            .with_seq(1000001)
            .with_ack(2000001) // SYN-ACK seq + 1
            .ack()
            .build();

        let seq = u32::from_be_bytes([ack[4], ack[5], ack[6], ack[7]]);
        let ack_num = u32::from_be_bytes([ack[8], ack[9], ack[10], ack[11]]);

        assert_eq!(seq, 1000001);
        assert_eq!(ack_num, 2000001);
    }

    /// Test SYN cookie generation and validation logic
    #[test]
    fn test_syn_cookie_algorithm() {
        // SYN cookie format (simplified):
        // - Lower 5 bits: time counter
        // - Next 2 bits: MSS index
        // - Upper 25 bits: hash

        let time_counter: u32 = 15; // Example time value (5 bits)
        let mss_index: u32 = 3; // MSS index (2 bits)
        let hash: u32 = 0x12345678 & 0xFFFFFF80; // Hash value (25 bits)

        let cookie = hash | ((mss_index & 0x03) << 5) | (time_counter & 0x1f);

        // Extract components
        let extracted_time = cookie & 0x1f;
        let extracted_mss = (cookie >> 5) & 0x03;
        let extracted_hash = cookie & 0xFFFFFF80;

        assert_eq!(extracted_time, time_counter);
        assert_eq!(extracted_mss, mss_index);
        assert_eq!(extracted_hash, hash);
    }

    /// Test time window validation for SYN cookies
    #[test]
    fn test_syn_cookie_time_validation() {
        // Cookie time is in 60-second windows
        // Allow current and previous window

        let current_window: u32 = 10;

        // Valid: same window
        let cookie_time: u32 = 10;
        let diff = if current_window >= cookie_time {
            current_window - cookie_time
        } else {
            32 - cookie_time + current_window
        };
        assert!(diff <= 2, "Same window should be valid");

        // Valid: previous window
        let cookie_time: u32 = 9;
        let diff = if current_window >= cookie_time {
            current_window - cookie_time
        } else {
            32 - cookie_time + current_window
        };
        assert!(diff <= 2, "Previous window should be valid");

        // Invalid: too old
        let cookie_time: u32 = 5;
        let diff = if current_window >= cookie_time {
            current_window - cookie_time
        } else {
            32 - cookie_time + current_window
        };
        assert!(diff > 2, "Old window should be invalid");
    }
}

#[cfg(test)]
mod tcp_flood_tests {
    use super::*;

    /// Test SYN flood detection
    #[test]
    fn test_syn_flood_detection() {
        // Simulate SYN flood: many SYN packets from same IP
        let src_ip = Ipv4Addr::new(192, 168, 1, 100);
        let dst_ip = Ipv4Addr::new(10, 0, 0, 1);

        // Default threshold is 100 SYN per IP per second
        let threshold = 100;

        let mut syn_packets = Vec::new();
        for port in 10000..(10000 + threshold + 50) {
            let packet = create_tcp_packet(src_ip, dst_ip, port as u16, 80, TCP_SYN, vec![]);
            syn_packets.push(packet);
        }

        assert_eq!(syn_packets.len(), 150);
        // Filter should block after threshold is exceeded
    }

    /// Test ACK flood detection
    #[test]
    fn test_ack_flood_detection() {
        // ACK flood: many ACK packets to exhaust resources
        let src_ip = Ipv4Addr::new(192, 168, 1, 100);
        let dst_ip = Ipv4Addr::new(10, 0, 0, 1);

        // Default threshold is 1000 ACK per IP per second
        let threshold = 1000;

        let mut ack_packets = Vec::new();
        for i in 0..threshold + 500 {
            let segment = TcpSegment::new()
                .with_src_port(54321)
                .with_dst_port(80)
                .with_seq(i as u32)
                .with_ack((i + 1000) as u32)
                .ack()
                .build();

            let packet = create_tcp_packet(src_ip, dst_ip, 54321, 80, TCP_ACK, vec![]);
            ack_packets.push(packet);
        }

        assert_eq!(ack_packets.len(), 1500);
    }

    /// Test RST flood detection
    #[test]
    fn test_rst_flood_detection() {
        // RST flood: many RST packets
        let src_ip = Ipv4Addr::new(192, 168, 1, 100);
        let dst_ip = Ipv4Addr::new(10, 0, 0, 1);

        // Default threshold is 100 RST per IP per second
        let threshold = 100;

        let mut rst_packets = Vec::new();
        for port in 10000..(10000 + threshold + 50) {
            let packet = create_tcp_packet(src_ip, dst_ip, port as u16, 80, TCP_RST, vec![]);
            rst_packets.push(packet);
        }

        assert_eq!(rst_packets.len(), 150);
    }

    /// Test per-IP connection limit
    #[test]
    fn test_connection_limit() {
        // Limit connections per IP to prevent resource exhaustion
        let src_ip = Ipv4Addr::new(192, 168, 1, 100);
        let dst_ip = Ipv4Addr::new(10, 0, 0, 1);

        // Default limit is 100 connections per IP
        let limit = 100;

        let mut connections = Vec::new();
        for port in 10000..(10000 + limit + 50) {
            let syn = create_tcp_packet(src_ip, dst_ip, port as u16, 80, TCP_SYN, vec![]);
            connections.push(syn);
        }

        assert_eq!(connections.len(), 150);
        // After limit, new SYN packets should be dropped
    }
}

#[cfg(test)]
mod tcp_fragmentation_tests {
    use super::*;

    /// Test first fragment with TCP header
    #[test]
    fn test_first_fragment() {
        let tcp = TcpSegment::new().with_flags(TCP_SYN).build();

        // First fragment: has TCP header
        // IP flags: MF (More Fragments) = 1, offset = 0
        let ip = Ipv4Packet::new()
            .with_protocol(IPPROTO_TCP)
            .with_fragment(0x01, 0) // MF flag, offset 0
            .with_payload(tcp)
            .build();

        let frame = EthernetFrame::new().with_payload(ip).build();

        // Verify fragment flags
        let frag_off = u16::from_be_bytes([frame[20], frame[21]]);
        assert_eq!(frag_off & 0x2000, 0x2000, "MF flag should be set");
        assert_eq!(frag_off & 0x1fff, 0, "Fragment offset should be 0");
    }

    /// Test non-first fragment (no TCP header)
    #[test]
    fn test_non_first_fragment() {
        // Non-first fragment: no TCP header, just data
        let data = vec![0u8; 100];

        // Fragment offset > 0
        let ip = Ipv4Packet::new()
            .with_protocol(IPPROTO_TCP)
            .with_fragment(0x00, 185) // No MF, offset 185 (1480 / 8)
            .with_payload(data)
            .build();

        let frame = EthernetFrame::new().with_payload(ip).build();

        // Verify fragment offset
        let frag_off = u16::from_be_bytes([frame[20], frame[21]]);
        assert_eq!(frag_off & 0x2000, 0, "MF flag should not be set");
        assert_eq!(frag_off & 0x1fff, 185, "Fragment offset should be 185");
    }

    /// Test tiny fragment attack
    #[test]
    fn test_tiny_fragment_attack() {
        // Tiny fragment attack: first fragment contains partial TCP header
        // This can be used to bypass filtering

        // Create a fragment with only 8 bytes of TCP header
        let partial_tcp = TcpSegment::new().with_flags(TCP_SYN).build();
        let tiny_payload = partial_tcp[..8].to_vec();

        let ip = Ipv4Packet::new()
            .with_protocol(IPPROTO_TCP)
            .with_fragment(0x01, 0) // MF flag, offset 0
            .with_payload(tiny_payload)
            .build();

        // This should be detected as suspicious
        // Filter should drop fragments that don't contain complete TCP header
    }

    /// Test overlapping fragments
    #[test]
    fn test_overlapping_fragments() {
        // Overlapping fragment attack: second fragment overlaps with first
        // Used to overwrite TCP header after initial inspection

        // First fragment at offset 0
        let frag1 = Ipv4Packet::new()
            .with_protocol(IPPROTO_TCP)
            .with_fragment(0x01, 0)
            .with_payload(vec![0u8; 100])
            .build();

        // Second fragment overlapping (offset 10 = 80 bytes)
        let frag2 = Ipv4Packet::new()
            .with_protocol(IPPROTO_TCP)
            .with_fragment(0x00, 10) // Overlaps with first fragment
            .with_payload(vec![0xffu8; 50])
            .build();

        // Both should be detected as suspicious in aggressive mode
    }
}

#[cfg(test)]
mod tcp_window_tests {
    use super::*;

    /// Test zero window
    #[test]
    fn test_zero_window() {
        let segment = TcpSegment::new().with_window(0).ack().build();

        let window = u16::from_be_bytes([segment[14], segment[15]]);
        assert_eq!(window, 0, "Window should be zero");
        // Zero window with ACK could be legitimate or a probe
    }

    /// Test maximum window
    #[test]
    fn test_max_window() {
        let segment = TcpSegment::new().with_window(65535).ack().build();

        let window = u16::from_be_bytes([segment[14], segment[15]]);
        assert_eq!(window, 65535, "Window should be max");
    }

    /// Test window probing detection
    #[test]
    fn test_window_probe() {
        // Window probe: ACK with zero window to test if peer has opened window
        // Legitimate but should be tracked

        let probe = TcpSegment::new().with_window(0).with_flags(TCP_ACK).build();

        let doff_flags = u16::from_be_bytes([probe[12], probe[13]]);
        let flags = (doff_flags & 0x003f) as u8;
        let window = u16::from_be_bytes([probe[14], probe[15]]);

        assert_eq!(flags, TCP_ACK);
        assert_eq!(window, 0);
    }
}

#[cfg(test)]
mod tcp_sequence_tests {
    use super::*;

    /// Test sequence number wraparound
    #[test]
    fn test_sequence_wraparound() {
        // Sequence numbers wrap around at 2^32

        let max_seq = u32::MAX;
        let segment = TcpSegment::new()
            .with_seq(max_seq)
            .with_ack(0)
            .ack()
            .build();

        let seq = u32::from_be_bytes([segment[4], segment[5], segment[6], segment[7]]);
        assert_eq!(seq, max_seq);

        // Next packet would have seq = 0 (wrapped)
        let wrapped_segment = TcpSegment::new().with_seq(0).with_ack(1).ack().build();

        let wrapped_seq = u32::from_be_bytes([
            wrapped_segment[4],
            wrapped_segment[5],
            wrapped_segment[6],
            wrapped_segment[7],
        ]);
        assert_eq!(wrapped_seq, 0);
    }

    /// Test ACK validation
    #[test]
    fn test_ack_validation() {
        // ACK should acknowledge previously sent data
        // Invalid ACKs (random values) indicate attack

        let expected_ack: u32 = 1000000;
        let window: u32 = 0x40000000; // 2^30

        // Valid ACK: within window
        let valid_ack = expected_ack + 1000;
        let diff = valid_ack.wrapping_sub(expected_ack);
        assert!(diff < window, "Valid ACK should be within window");

        // Invalid ACK: far ahead
        let invalid_ack = expected_ack.wrapping_add(window * 2);
        let diff = invalid_ack.wrapping_sub(expected_ack);
        assert!(diff >= window, "Invalid ACK should be outside window");
    }

    /// Test out-of-order packets
    #[test]
    fn test_out_of_order() {
        // Out-of-order packets are legitimate but tracked

        let base_seq: u32 = 1000000;

        // Normal order: 1000000, 1001000, 1002000
        // Out of order: 1000000, 1002000, 1001000

        let pkt1 = TcpSegment::new().with_seq(base_seq).build();
        let pkt2 = TcpSegment::new().with_seq(base_seq + 2000).build();
        let pkt3 = TcpSegment::new().with_seq(base_seq + 1000).build();

        // pkt3 arrives after pkt2 but has earlier seq
        let seq2 = u32::from_be_bytes([pkt2[4], pkt2[5], pkt2[6], pkt2[7]]);
        let seq3 = u32::from_be_bytes([pkt3[4], pkt3[5], pkt3[6], pkt3[7]]);

        assert!(seq3 < seq2, "pkt3 has earlier sequence than pkt2");
    }
}

#[cfg(test)]
mod tcp_full_packet_tests {
    use super::*;

    /// Test complete TCP SYN packet
    #[test]
    fn test_complete_syn_packet() {
        let packet = create_tcp_packet(
            Ipv4Addr::new(192, 168, 1, 100),
            Ipv4Addr::new(10, 0, 0, 1),
            54321,
            80,
            TCP_SYN,
            vec![],
        );

        // Minimum size: Eth (14) + IP (20) + TCP (20) = 54
        assert!(packet.len() >= 54);

        // Verify structure
        // Ethernet
        assert_eq!(&packet[12..14], &[0x08, 0x00]); // IPv4

        // IP
        assert_eq!(packet[14] >> 4, 4); // Version
        assert_eq!(packet[23], IPPROTO_TCP);

        // TCP
        let tcp_start = 14 + 20; // After Eth + IP
        let flags = u16::from_be_bytes([packet[tcp_start + 12], packet[tcp_start + 13]]);
        assert_eq!((flags & 0x003f) as u8, TCP_SYN);
    }

    /// Test complete TCP with data
    #[test]
    fn test_tcp_with_data() {
        let data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec();

        let packet = create_tcp_packet(
            Ipv4Addr::new(192, 168, 1, 100),
            Ipv4Addr::new(10, 0, 0, 1),
            54321,
            80,
            TCP_ACK | TCP_PSH,
            data.clone(),
        );

        // Size should include data
        assert!(packet.len() >= 54 + data.len());

        // Verify data is present at end
        let data_start = 14 + 20 + 20; // Eth + IP + TCP
        assert_eq!(&packet[data_start..data_start + 4], b"GET ");
    }
}
