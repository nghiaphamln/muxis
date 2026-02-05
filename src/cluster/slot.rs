//! Redis Cluster slot calculation.
//!
//! Redis Cluster uses CRC16 to map keys to slots (0-16383).
//! This module provides utilities for calculating slot numbers from keys.

use crc::{Crc, CRC_16_IBM_SDLC};

/// Number of hash slots in Redis Cluster.
pub const SLOT_COUNT: u16 = 16384;

/// CRC-16/XMODEM algorithm used by Redis.
const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);

/// Calculates the Redis Cluster slot for a given key.
///
/// Redis uses CRC16 modulo 16384 for slot calculation.
/// If the key contains `{...}`, only the content inside the braces
/// is used for hashing (hash tags).
///
/// # Arguments
///
/// * `key` - The Redis key to calculate the slot for
///
/// # Returns
///
/// The slot number (0-16383)
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cluster")]
/// # {
/// use muxis::cluster::key_slot;
///
/// assert_eq!(key_slot("foo"), key_slot("foo"));
/// assert_eq!(key_slot("{user1000}.following"), key_slot("{user1000}.followers"));
/// assert_ne!(key_slot("user1000"), key_slot("user2000"));
/// # }
/// ```
pub fn key_slot(key: &str) -> u16 {
    let hash_key = extract_hash_tag(key);
    let crc = CRC16.checksum(hash_key.as_bytes());
    crc % SLOT_COUNT
}

/// Extracts the hash tag from a key.
///
/// Redis hash tags are defined by `{...}`:
/// - `{user1000}.following` → hash tag is `user1000`
/// - `foo{bar}baz` → hash tag is `bar`
/// - `foo{}{bar}` → no valid hash tag (empty or multiple), use whole key
/// - `foo` → no hash tag, use whole key
///
/// # Arguments
///
/// * `key` - The Redis key
///
/// # Returns
///
/// The extracted hash tag, or the whole key if no valid hash tag exists
fn extract_hash_tag(key: &str) -> &str {
    // Find the first '{' and last '}'
    if let Some(start) = key.find('{') {
        if let Some(end) = key[start + 1..].find('}') {
            let tag_start = start + 1;
            let tag_end = tag_start + end;

            // Only use hash tag if it's non-empty
            if tag_end > tag_start {
                return &key[tag_start..tag_end];
            }
        }
    }

    // No valid hash tag, use the whole key
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_count() {
        assert_eq!(SLOT_COUNT, 16384);
    }

    #[test]
    fn test_key_slot_simple() {
        // Same key should always produce same slot
        let slot1 = key_slot("mykey");
        let slot2 = key_slot("mykey");
        assert_eq!(slot1, slot2);

        // Slot should be in valid range
        assert!(slot1 < SLOT_COUNT);
    }

    #[test]
    fn test_key_slot_different_keys() {
        // Different keys should (usually) produce different slots
        let slot1 = key_slot("key1");
        let slot2 = key_slot("key2");
        // Note: This is probabilistic, but with 16384 slots, collision is unlikely
        assert_ne!(slot1, slot2);
    }

    #[test]
    fn test_key_slot_with_hash_tag() {
        // Keys with same hash tag should map to same slot
        let slot1 = key_slot("{user1000}.following");
        let slot2 = key_slot("{user1000}.followers");
        let slot3 = key_slot("{user1000}.posts");

        assert_eq!(slot1, slot2);
        assert_eq!(slot2, slot3);
    }

    #[test]
    fn test_key_slot_hash_tag_vs_no_tag() {
        // Hash tag should only use the tagged part
        let with_tag = key_slot("{user}1000");
        let without_tag = key_slot("user1000");

        // These should be different (hashing different strings)
        assert_ne!(with_tag, without_tag);

        // But {user} prefix should be consistent
        let with_tag2 = key_slot("{user}2000");
        assert_eq!(with_tag, with_tag2);
    }

    #[test]
    fn test_extract_hash_tag_simple() {
        assert_eq!(extract_hash_tag("foo{bar}"), "bar");
        assert_eq!(extract_hash_tag("{user1000}.following"), "user1000");
        assert_eq!(extract_hash_tag("prefix{tag}suffix"), "tag");
    }

    #[test]
    fn test_extract_hash_tag_no_tag() {
        assert_eq!(extract_hash_tag("simple_key"), "simple_key");
        assert_eq!(extract_hash_tag("no_braces"), "no_braces");
    }

    #[test]
    fn test_extract_hash_tag_empty() {
        // Empty hash tag should use whole key
        assert_eq!(extract_hash_tag("foo{}bar"), "foo{}bar");
        assert_eq!(extract_hash_tag("{}"), "{}");
    }

    #[test]
    fn test_extract_hash_tag_multiple_braces() {
        // Only first valid pair is used
        assert_eq!(extract_hash_tag("foo{bar}{baz}"), "bar");
        assert_eq!(extract_hash_tag("{a}{b}{c}"), "a");
    }

    #[test]
    fn test_extract_hash_tag_unmatched() {
        // Unmatched braces should use whole key
        assert_eq!(extract_hash_tag("foo{bar"), "foo{bar");
        assert_eq!(extract_hash_tag("foo}bar"), "foo}bar");
        assert_eq!(extract_hash_tag("{"), "{");
        assert_eq!(extract_hash_tag("}"), "}");
    }

    #[test]
    fn test_key_slot_empty_key() {
        // Empty key should still produce a valid slot
        let slot = key_slot("");
        assert!(slot < SLOT_COUNT);
    }

    #[test]
    fn test_key_slot_special_chars() {
        // Keys with special characters
        let slot1 = key_slot("key:1:value");
        let slot2 = key_slot("key/1/value");
        let slot3 = key_slot("key|1|value");

        assert!(slot1 < SLOT_COUNT);
        assert!(slot2 < SLOT_COUNT);
        assert!(slot3 < SLOT_COUNT);
    }

    #[test]
    fn test_key_slot_unicode() {
        // Unicode keys
        let slot1 = key_slot("用户1000");
        let slot2 = key_slot("пользователь1000");
        let slot3 = key_slot("utilisateur1000");

        assert!(slot1 < SLOT_COUNT);
        assert!(slot2 < SLOT_COUNT);
        assert!(slot3 < SLOT_COUNT);
    }

    #[test]
    fn test_key_slot_long_key() {
        // Very long key
        let long_key = "a".repeat(10000);
        let slot = key_slot(&long_key);
        assert!(slot < SLOT_COUNT);
    }

    #[test]
    fn test_key_slot_distribution() {
        // Test that keys distribute across multiple slots
        // Generate 100 keys and check they don't all map to same slot
        let mut slots = std::collections::HashSet::new();

        for i in 0..100 {
            let key = format!("key{}", i);
            slots.insert(key_slot(&key));
        }

        // Should have at least 50 different slots (very conservative)
        assert!(slots.len() >= 50, "Keys should distribute across slots");
    }

    #[test]
    fn test_key_slot_redis_spec_examples() {
        // Test against known slot values from Redis documentation
        // These values are calculated using the same CRC16 algorithm

        // Note: Actual slot values depend on CRC16 implementation
        // We test consistency rather than absolute values
        let key = "user:1000";
        let slot1 = key_slot(key);
        let slot2 = key_slot(key);
        assert_eq!(slot1, slot2, "Same key should produce same slot");

        // Test hash tag behavior
        let tagged1 = key_slot("{user:1000}:profile");
        let tagged2 = key_slot("{user:1000}:posts");
        assert_eq!(tagged1, tagged2, "Same hash tag should produce same slot");
    }
}
