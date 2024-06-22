// a symbol is a item in the set
//
// A CodedSymbol can be peeled when the count is 1 or -1 and the hash matches
//
// To better fit with the Rust ecoststem, I wonder if it would be better to require the symbol to
// be encodeable + decodable (to a fixed width byte array) rather than specify the xor function
// below
// We need this to be able to send the codedSymbols over the network.
// The xor of a bytearray is trivial
use std::fmt::Debug;
use std::marker::PhantomData;
use std::hash::{DefaultHasher, Hash, Hasher};

pub trait Symbol: Clone + Debug {
    const BYTE_ARRAY_LENGTH: usize;
    // fn encode_to_bytes(&self) -> [u8; Self::BYTE_ARRAY_LENGTH];
    // fn decode_from_bytes(bytes: &[u8; Self::BYTE_ARRAY_LENGTH]) -> Self;
    fn encode_to_bytes(&self) -> Vec<u8>;
    fn decode_from_bytes(bytes: &Vec<u8>) -> Self;

    fn hash_(&self) -> u64 {
        let encoded = self.encode_to_bytes();
        let mut hasher = DefaultHasher::new();
        encoded.hash(&mut hasher);
        hasher.finish()
    }

    // fn empty() -> Self;

    // // XOR returns t ^ t2, where t is the method receiver. XOR is allowed to
    // // modify the method receiver. Although the method is called XOR (because
    // // the bitwise exclusive-or operation is a valid group operation for groups
    // // of fixed-length bit strings), it can implement any operation that
    // // satisfy the aforementioned properties.
    // fn xor(&mut self, other: &Self) -> Self;

    // // hash_ returns the hash of the method receiver. It must not modify the
    // // method receiver. It must not be homomorphic over the group operation.
    // // That is, the probability that
    // //   (a ^ b).hash() == a.hash() ^ b.hash()
    // // must be negligible. Here, ^ is the group operation on the left-hand
    // // side, and bitwise exclusive-or on the right side.
    // fn hash_(&self) -> u64;
}

// coded symbol produced by a Rateless IBLT encoder.
// It might be good to store the encoded symbol rather than the symbol itself
#[derive(Clone, Debug)]
pub struct CodedSymbol<T: Symbol> {
    // pub symbol: T,
    _marker: PhantomData<T>,
    pub sum: Vec<u8>,
    pub hash: u64,
    pub count: i64,
}

impl<T: Symbol> CodedSymbol<T> {
    pub fn new() -> Self {
        // let symbol = T::empty();
        let sum = vec![0u8; T::BYTE_ARRAY_LENGTH];
        let hash = 0;
        let count = 0;
        CodedSymbol {
            // symbol,
            _marker: PhantomData,
            sum,
            hash,
            count,
        }
    }
}

// When peeling, codedSymbols with a count of 1 are present only locally
// and codedSymbols with a count of -1 were present only in the remote set

#[derive(PartialEq, Eq, Clone)]
pub enum PeelableResult<T: Symbol> {
    Local(T),
    Remote(T),
    NotPeelable,
}

#[derive(Clone)]
pub enum Direction {
    Add,
    Remove,
}

impl<T: Symbol> CodedSymbol<T> {
    //It might be nice to split this into an 'add' and 'remove'
    pub fn apply(&mut self, s: &T, direction: Direction) {

        assert_eq!(self.sum.len(), T::BYTE_ARRAY_LENGTH, "self.sum must have the length specified by T::BYTE_ARRAY_LENGTH.");
        let encoded_s = s.encode_to_bytes();

        assert_eq!(encoded_s.len(), T::BYTE_ARRAY_LENGTH, "encoded_s must have the length specified by T::BYTE_ARRAY_LENGTH.");

        self.sum = self.sum.iter()
            .zip(encoded_s.iter())
            .map(|(x, y)| x ^ y)
            .collect();

        // self.symbol = self.symbol.xor(&s);
        self.hash ^= s.hash_();
        match direction {
            Direction::Add => self.count += 1,
            Direction::Remove => self.count -= 1,
        };
    }

    //used by the encoder to join two vectors of codedSymbols together produced from two distinct sets
    //The results are only valid if there were no duplicates between the original sets
    pub fn combine(&self, b: &CodedSymbol<T>) -> CodedSymbol<T> {

        assert_eq!(self.sum.len(), T::BYTE_ARRAY_LENGTH, "self.sum must have the length specified by T::BYTE_ARRAY_LENGTH.");
        assert_eq!(b.sum.len(), T::BYTE_ARRAY_LENGTH, "encoded_s must have the length specified by T::BYTE_ARRAY_LENGTH.");

        let mut new_coded_symbol = self.clone();

        // new_coded_symbol.symbol = new_coded_symbol.symbol.xor(&b.symbol);
        new_coded_symbol.hash ^= b.hash;
        new_coded_symbol.count += b.count;

        new_coded_symbol.sum = self.sum.iter()
            .zip(b.sum.iter())
            .map(|(x, y)| x ^ y)
            .collect();

        new_coded_symbol
    }

    pub fn collapse(&self, b: &CodedSymbol<T>) -> CodedSymbol<T> {
        assert_eq!(self.sum.len(), T::BYTE_ARRAY_LENGTH, "self.sum must have the length specified by T::BYTE_ARRAY_LENGTH.");
        assert_eq!(b.sum.len(), T::BYTE_ARRAY_LENGTH, "encoded_s must have the length specified by T::BYTE_ARRAY_LENGTH.");

        let mut new_coded_symbol = self.clone();

        // new_coded_symbol.symbol = new_coded_symbol.symbol.xor(&b.symbol);
        new_coded_symbol.hash ^= b.hash;
        new_coded_symbol.count -= b.count;

        new_coded_symbol.sum = self.sum.iter()
            .zip(b.sum.iter())
            .map(|(x, y)| x ^ y)
            .collect();

        new_coded_symbol
    }

    // this does not modify the CodedSymbol
    pub fn is_peelable(&self) -> bool {
        if self.count == 1 || self.count == -1 {
            if self.hash == T::decode_from_bytes(&self.sum).hash_() {
                return true;
            }
        }
        return false;
    }

    pub fn peel(&mut self) -> PeelableResult<T> {
        if self.is_peelable() {
            let return_result = if self.count == 1 {
                PeelableResult::Local(T::decode_from_bytes(&self.sum))
            } else {
                PeelableResult::Remote(T::decode_from_bytes(&self.sum))
            };

            *self = CodedSymbol::new();
            return return_result;
        }
        PeelableResult::NotPeelable
    }
    // same as peel, but does not modify the CodedSymbol
    pub fn peel_peek(&self) -> PeelableResult<T> {
        if self.is_peelable() {
            let return_result = if self.count == 1 {
                PeelableResult::Local(T::decode_from_bytes(&self.sum))
            } else {
                PeelableResult::Remote(T::decode_from_bytes(&self.sum))
            };

            return return_result;
        }
        PeelableResult::NotPeelable
    }
    pub fn is_empty(&self) -> bool {
        // if self.symbol != T::empty() {
        //     return false;
        // }
        if self.count != 0 {
            return false;
        }
        if self.hash != 0 {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::SimpleSymbol;

    #[test]
    fn test_symbol() {
        let symbol1 = SimpleSymbol { value: 42 };
        let symbol2 = SimpleSymbol { value: 100 };

        // let hash_symbol1 = symbol::HashedSymbol {
        //     symbol: symbol1.clone(),
        //     hash: symbol1.hash_(),
        // };
        // let hash_symbol2 = symbol::HashedSymbol {
        //     symbol: symbol2.clone(),
        //     hash: symbol2.hash_(),
        // };
        let mut coded_symbol = CodedSymbol::new();

        println!("0 is peelable {}", coded_symbol.is_peelable());
        assert_eq!(coded_symbol.is_peelable(), false);

        coded_symbol.apply(&symbol1, Direction::Add);
        println!("1 is peelable {}", coded_symbol.is_peelable());
        assert_eq!(coded_symbol.is_peelable(), true);

        coded_symbol.apply(&symbol2, Direction::Add);
        println!("2 is peelable {}", coded_symbol.is_peelable());
        assert_eq!(coded_symbol.is_peelable(), false);

        coded_symbol.apply(&symbol1, Direction::Remove);
        println!("3 is peelable {}", coded_symbol.is_peelable());
        assert_eq!(coded_symbol.is_peelable(), true);

        println!("CodedSymbol: {:?}", coded_symbol);

        let peeled_symbol = coded_symbol.peel();
        match peeled_symbol {
            PeelableResult::Local(symbol) => {
                println!("Peeled Local Symbol: {:?}", symbol);
                assert_eq!(symbol.value, symbol2.value);
            }
            PeelableResult::Remote(symbol) => {
                println!("Peeled Remote Symbol: {:?}", symbol);
                assert_eq!(symbol.value, symbol2.value);
            }
            PeelableResult::NotPeelable => {
                println!("No symbol to peel");
                assert!(false);
            }
        }

        // assert!(false);
    }
}

