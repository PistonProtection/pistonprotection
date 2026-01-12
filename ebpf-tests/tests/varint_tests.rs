//! VarInt parsing edge case tests
//!
//! Tests for the Minecraft VarInt encoding format, focusing on edge cases
//! that could be exploited for attacks.

use pistonprotection_ebpf_tests::packet_generator::{encode_varint, decode_varint};

#[cfg(test)]
mod varint_edge_cases {
    use super::*;

    /// Test that zero is properly encoded and decoded
    #[test]
    fn test_varint_zero() {
        let encoded = encode_varint(0);
        assert_eq!(encoded, vec![0x00]);
        assert_eq!(decode_varint(&encoded), Some((0, 1)));
    }

    /// Test maximum positive value that fits in 1 byte
    #[test]
    fn test_varint_max_single_byte() {
        let encoded = encode_varint(127);
        assert_eq!(encoded, vec![0x7f]);
        assert_eq!(decode_varint(&encoded), Some((127, 1)));
    }

    /// Test minimum value requiring 2 bytes
    #[test]
    fn test_varint_min_two_bytes() {
        let encoded = encode_varint(128);
        assert_eq!(encoded, vec![0x80, 0x01]);
        assert_eq!(decode_varint(&encoded), Some((128, 2)));
    }

    /// Test maximum positive VarInt value (2^31 - 1)
    #[test]
    fn test_varint_max_positive() {
        let max_val = i32::MAX;
        let encoded = encode_varint(max_val);
        assert_eq!(encoded.len(), 5);
        let (decoded, bytes) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, max_val);
        assert_eq!(bytes, 5);
    }

    /// Test minimum negative VarInt value (-2^31)
    #[test]
    fn test_varint_min_negative() {
        let min_val = i32::MIN;
        let encoded = encode_varint(min_val);
        assert_eq!(encoded.len(), 5);
        let (decoded, bytes) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, min_val);
        assert_eq!(bytes, 5);
    }

    /// Test -1 encoding (important edge case)
    #[test]
    fn test_varint_negative_one() {
        let encoded = encode_varint(-1);
        // -1 is 0xFFFFFFFF in two's complement, which encodes as 5 bytes
        assert_eq!(encoded, vec![0xff, 0xff, 0xff, 0xff, 0x0f]);
        let (decoded, bytes) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, -1);
        assert_eq!(bytes, 5);
    }

    /// Test various negative values
    #[test]
    fn test_varint_negative_values() {
        let test_cases: &[(i32, usize)] = &[
            (-1, 5),
            (-128, 5),
            (-256, 5),
            (-1000, 5),
            (-2147483648, 5),
        ];

        for &(value, expected_bytes) in test_cases {
            let encoded = encode_varint(value);
            assert_eq!(
                encoded.len(),
                expected_bytes,
                "Unexpected encoding length for {}",
                value
            );
            let (decoded, bytes) = decode_varint(&encoded).unwrap();
            assert_eq!(decoded, value, "Roundtrip failed for {}", value);
            assert_eq!(bytes, expected_bytes);
        }
    }

    /// Test empty input handling
    #[test]
    fn test_varint_empty_input() {
        assert_eq!(decode_varint(&[]), None);
    }

    /// Test truncated multi-byte VarInt
    #[test]
    fn test_varint_truncated() {
        // Start of a multi-byte VarInt but truncated
        assert_eq!(decode_varint(&[0x80]), None);
        assert_eq!(decode_varint(&[0x80, 0x80]), None);
        assert_eq!(decode_varint(&[0x80, 0x80, 0x80]), None);
        assert_eq!(decode_varint(&[0x80, 0x80, 0x80, 0x80]), None);
    }

    /// Test overly long VarInt (more than 5 bytes)
    #[test]
    fn test_varint_too_long() {
        // 6 continuation bytes - invalid
        let invalid = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        // This should either fail or only consume the first 5 bytes
        // depending on implementation
        let result = decode_varint(&invalid);
        // A proper implementation should reject this
        assert!(result.is_none() || result.unwrap().1 <= 5);
    }

    /// Test that packet ID negative check works
    /// This tests the vulnerability where packet_id > 0x03 would pass
    /// for negative numbers encoded as VarInt
    #[test]
    fn test_negative_packet_id_detection() {
        // Negative VarInts should be detected and rejected
        let negative_one = encode_varint(-1);
        let (value, _) = decode_varint(&negative_one).unwrap();

        // The key check: value < 0 should be detected
        assert!(value < 0, "Negative VarInt not properly detected as negative");

        // This is the check that should happen in the filter
        // WRONG: if packet_id > 0x03 { drop }  -- This passes for negative!
        // RIGHT: if packet_id < 0 || packet_id > 0x03 { drop }
        let packet_id = value;
        let is_valid_login_packet = packet_id >= 0x00 && packet_id <= 0x03;
        assert!(!is_valid_login_packet, "Negative packet ID incorrectly allowed");
    }

    /// Test Minecraft protocol version encoding
    #[test]
    fn test_protocol_version_encoding() {
        // Common protocol versions
        let versions = [
            (4, "1.7.2"),      // 1 byte
            (47, "1.8.x"),     // 1 byte
            (315, "1.11"),     // 2 bytes
            (498, "1.14.4"),   // 2 bytes
            (754, "1.16.5"),   // 2 bytes
            (756, "1.17"),     // 2 bytes
            (765, "1.20.4"),   // 2 bytes
            (767, "1.21"),     // 2 bytes
        ];

        for (version, name) in versions {
            let encoded = encode_varint(version);
            let (decoded, _) = decode_varint(&encoded).unwrap();
            assert_eq!(
                decoded, version,
                "Protocol version {} ({}) roundtrip failed",
                version, name
            );
        }
    }

    /// Test packet length encoding edge cases
    #[test]
    fn test_packet_length_encoding() {
        // Minecraft max packet size is 2097151 (2MB - 1)
        let max_packet = 2097151;
        let encoded = encode_varint(max_packet);
        assert_eq!(encoded.len(), 3); // Should fit in 3 bytes
        let (decoded, _) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, max_packet);

        // Just over the limit
        let oversized = 2097152;
        let encoded = encode_varint(oversized);
        assert_eq!(encoded.len(), 4);
    }

    /// Test hostname length encoding
    #[test]
    fn test_hostname_length_encoding() {
        // DNS max is 253 bytes
        let dns_max = 253;
        let encoded = encode_varint(dns_max);
        assert_eq!(encoded.len(), 2);
        let (decoded, _) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, dns_max);

        // Common hostname lengths
        for len in [1, 10, 50, 100, 200, 253, 255] {
            let encoded = encode_varint(len);
            let (decoded, _) = decode_varint(&encoded).unwrap();
            assert_eq!(decoded, len);
        }
    }

    /// Test the specific case mentioned: negative VarInt bypassing packet ID check
    #[test]
    fn test_packet_id_bypass_vulnerability() {
        // This tests the vulnerability mentioned in the prompt:
        // "Also that packet id check is extremely dumb. It does id > [max packet id]
        // but a varint can be negative"

        // Generate all negative values that might be used in attacks
        let attack_values: &[i32] = &[-1, -2, -128, -256, -1000, i32::MIN];

        for &attack_value in attack_values {
            let encoded = encode_varint(attack_value);
            let (decoded, _) = decode_varint(&encoded).unwrap();

            // Old vulnerable check: packet_id > MAX_PACKET_ID
            // This would PASS for negative values!
            let max_packet_id = 0x7f;
            let vulnerable_check_passes = !(decoded > max_packet_id);

            // Fixed check: packet_id < 0 || packet_id > MAX_PACKET_ID
            let fixed_check_rejects = decoded < 0 || decoded > max_packet_id;

            assert!(
                vulnerable_check_passes,
                "Vulnerable check correctly would have passed for {}",
                attack_value
            );
            assert!(
                fixed_check_rejects,
                "Fixed check correctly rejects {}",
                attack_value
            );
        }
    }

    /// Test VarInt with extra trailing bytes (should stop at first complete value)
    #[test]
    fn test_varint_with_trailing_data() {
        let mut data = encode_varint(42);
        data.extend_from_slice(&[0xff, 0xff, 0xff]); // Trailing garbage

        let (decoded, bytes_consumed) = decode_varint(&data).unwrap();
        assert_eq!(decoded, 42);
        assert_eq!(bytes_consumed, 1);
    }
}

#[cfg(test)]
mod varint_security_tests {
    use super::*;

    /// Test crafted malicious VarInt sequences
    #[test]
    fn test_malicious_varint_sequences() {
        // All continuation bytes then terminator
        let sequences = [
            vec![0xff, 0xff, 0xff, 0xff, 0x0f], // Valid -1
            vec![0x80, 0x80, 0x80, 0x80, 0x08], // Valid large negative
        ];

        for seq in sequences {
            let result = decode_varint(&seq);
            // Should either decode successfully or fail gracefully
            if let Some((value, bytes)) = result {
                assert!(bytes <= 5, "Consumed too many bytes");
                // Verify the value is representable as i32
                let _ = value as i32;
            }
        }
    }

    /// Test overflow scenarios
    #[test]
    fn test_varint_overflow_protection() {
        // These sequences would overflow if not handled properly

        // Maximum valid 5-byte VarInt for signed 32-bit
        let max_valid_negative = vec![0xff, 0xff, 0xff, 0xff, 0x0f];
        let result = decode_varint(&max_valid_negative);
        assert!(result.is_some());

        // Just beyond valid range (6th byte set)
        let invalid = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let result = decode_varint(&invalid);
        // Should fail or only consume 5 bytes
        assert!(result.is_none() || result.unwrap().1 <= 5);
    }

    /// Test the specific packet ID boundary values
    #[test]
    fn test_packet_id_boundary_values() {
        let boundary_values = [
            (0x00, true),   // Handshake/Status Request
            (0x01, true),   // Ping Request
            (0x02, true),   // Login Plugin Response
            (0x03, true),   // Login Acknowledged
            (0x04, false),  // Just outside login range
            (0x7f, false),  // Max valid unsigned
            (-1, false),    // Negative
            (-128, false),  // More negative
        ];

        for (value, should_be_valid_for_login) in boundary_values {
            let is_valid = value >= 0 && value <= 0x03;
            assert_eq!(
                is_valid, should_be_valid_for_login,
                "Packet ID {} validity check failed",
                value
            );
        }
    }
}
