//! The output sent to a terminal to measure how it takes it in. Every load is
//! deterministic: the same seed gives the same bytes, so all terminals get
//! exactly the same work.

mod random;
mod scrub;
mod units;

pub use random::{fnv1a64, XorShift64};
pub use scrub::scrub_paths;

use crate::metric::Load;

/// Exactly `bytes` bytes of valid UTF-8. A load is made of whole units (a line,
/// a progress bar); the last one is cut between two characters and what is left
/// over is filled with spaces.
pub fn generate(load: Load, bytes: usize, seed: u64) -> Vec<u8> {
    let mut rng = XorShift64::new(seed);
    let mut text = String::with_capacity(bytes + 16 * 1024);
    let mut index = 0;
    while text.len() < bytes {
        units::push(load, index, &mut rng, &mut text);
        index += 1;
    }
    let mut end = bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
    let mut out = text.into_bytes();
    out.resize(bytes, b' ');
    out
}

#[cfg(test)]
mod tests;
