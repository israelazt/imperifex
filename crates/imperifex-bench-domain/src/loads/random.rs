//! A small pseudorandom generator and a hash of our own (decision D8): the same
//! on every machine and in every version, so a load never changes by itself.

/// Used when the seed asked for is zero, which xorshift could never leave.
const SEED_FOR_ZERO: u64 = 0x9E37_79B9_7F4A_7C15;

/// Marsaglia's xorshift with shifts 13, 7 and 17.
#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { SEED_FOR_ZERO } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// A number under `limit`, which must not be zero.
    pub(super) fn below(&mut self, limit: u64) -> u64 {
        self.next_u64() % limit
    }
}

/// The 64 bit FNV-1a hash.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01B3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_gives_the_published_values() {
        assert_eq!(fnv1a64(b""), 0xCBF2_9CE4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xAF63_DC4C_8601_EC8C);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_F739_67E8);
    }

    #[test]
    fn the_same_seed_gives_the_same_numbers_and_another_gives_others() {
        let take = |seed| {
            let mut rng = XorShift64::new(seed);
            [rng.next_u64(), rng.next_u64(), rng.next_u64()]
        };
        assert_eq!(take(1), take(1));
        assert_ne!(take(1), take(2));
    }

    #[test]
    fn a_seed_of_zero_does_not_get_stuck() {
        let mut rng = XorShift64::new(0);
        assert_ne!(rng.next_u64(), 0);
        assert_ne!(rng.next_u64(), rng.next_u64());
    }
}
