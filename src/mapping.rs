use crate::symbol::Symbol;
use std::f64;

pub struct RandomMapping {
    prng: u64,
    last_idx: u64,
}

impl Iterator for RandomMapping {
    type Item = usize;

    // Update the pseudo random state and calculate the next index.
    fn next(&mut self) -> Option<usize> {
        self.prng = self.prng.wrapping_mul(0xda942042e4dd58b5);
        let r = self.prng;

        //2^32
        let tp32: f64 = (1u64 << 32) as f64;

        // diff to next index
        let diff = (self.last_idx as f64 + 1.5) * (tp32 / (r as f64 + 1.0).sqrt() - 1.0);

        let index_to_return = self.last_idx;
        self.last_idx += diff.ceil() as u64;

        Some(index_to_return as usize)
    }
}

impl RandomMapping {
    pub fn new<T: Symbol>(given_symbol: &T) -> Self {
        let prng = given_symbol.hash_();
        RandomMapping { prng, last_idx: 0 }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::SimpleSymbol;
    use crate::*;

    #[test]
    fn test_mapping() {
        let rm = RandomMapping::new(&SimpleSymbol { value: 1 });

        for index in rm.take(10) {
            println!("{}", index);
        }
        let rm = RandomMapping::new(&SimpleSymbol { value: 2 });
        for index in rm.take(10) {
            println!("{}", index);
        }

        let rm = RandomMapping::new(&SimpleSymbol { value: 2 });

        //combining take_while and filter can give us the indexes that land in a range
        //helpful if we are computing the coded symbols in a block
        let below_100: Vec<usize> = rm.take_while(|&x| x <= 100).filter(|&x| x > 30).collect();
        println!("{:?}", below_100);
        // assert!(false);
    }
}
