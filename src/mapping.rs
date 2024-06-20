use std::f64;
use crate::symbol::Symbol;

pub struct RandomMapping {
    prng: u64,
    last_idx: u64,
}

impl Iterator for RandomMapping {
    type Item = u64;

    // Update the pseudo random state and calculate the next index.
    fn next(&mut self) -> Option<u64> {
        self.prng = self.prng.wrapping_mul(0xda942042e4dd58b5);
        let r = self.prng;

        //2^32
        let tp32: f64 = (1u64 << 32) as f64;

        // diff to next index
        let diff = (self.last_idx as f64 + 1.5) * (tp32 / (r as f64 + 1.0).sqrt() - 1.0);

        let index_to_return = self.last_idx;
        self.last_idx += diff.ceil() as u64;

        Some(index_to_return)
    }
}

impl RandomMapping {
    pub fn new<T: Symbol>(given_symbol: &T) -> Self {
        let prng = given_symbol.hash_();
        RandomMapping { prng, last_idx: 0 }
    }
}
