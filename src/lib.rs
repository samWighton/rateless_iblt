mod encoder;
mod mapping;
mod symbol;

pub use encoder::{RatelessIBLT, UnmanagedRatelessIBLT};
pub use mapping::RandomMapping;
pub use symbol::{Symbol, CodedSymbol};

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::hash::Hash;

    // Example implementation of a struct that implements the 'Symbol' trait
    // This is used in tests in other modules
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct SimpleSymbol {
        pub value: u64,
    }

    impl symbol::Symbol for SimpleSymbol {
        const BYTE_ARRAY_LENGTH: usize = 8;
        // fn xor(&mut self, other: &Self) -> Self {
        //     Self {
        //         value: self.value ^ other.value,
        //     }
        // }
        // fn hash_(&self) -> u64 {
        //     let mut hasher = DefaultHasher::new();
        //     self.value.hash(&mut hasher);
        //     hasher.finish()
        // }
        // fn empty() -> Self {
        //     SimpleSymbol { value: 0 }
        // }
        fn encode_to_bytes(&self) -> Vec<u8> {
            let mut buffer = vec![0u8; SimpleSymbol::BYTE_ARRAY_LENGTH];
            buffer[0..8].copy_from_slice(&self.value.to_le_bytes());
            buffer
        }
        fn decode_from_bytes(buffer: &Vec<u8>) -> Self {
            let value = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
            SimpleSymbol { value }
        }
    }
}
