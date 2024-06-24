use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;

/// A symbol is an item in the set
pub trait Symbol: Clone + Debug {
    const BYTE_ARRAY_LENGTH: usize;

    /// The Symbol trait only requires that the type can be encoded to a fixed number of bytes.
    /// You just need to know the size of the byte array that will be produced and then set BYTE_ARRAY_LENGTH to match.
    /// I recommend using a serialization library like bincode.
    fn encode_to_bytes(&self) -> Vec<u8>;
    fn decode_from_bytes(bytes: &Vec<u8>) -> Self;

    /// hash_() calculates the hash of the symbol.
    /// This implementation can be overridden if needed.
    fn hash_(&self) -> u64 {
        let encoded = self.encode_to_bytes();
        let mut hasher = DefaultHasher::new();
        encoded.hash(&mut hasher);
        hasher.finish()
    }
}

/// A RIBLT is an infinite sequence of CodedSymbols
///
/// The 'sum' field is the XOR of the symbols encoded into this CodedSymbol
///
/// The 'hash' field is the XOR of the hashes of the symbols encoded into this CodedSymbol
///
/// The 'count' field is the number of local symbols minus the number of remote symbols
///
/// The '_marker' phantom field is used to allow us to associate the CodedSymbol with a specific Symbol type.
/// The type T is used by implemented methods to know what type of Symbol is encoded in the CodedSymbol.
///
/// A CodedSymbol can be peeled when the count is 1 or -1 and the hash matches
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodedSymbol<T: Symbol> {
    _marker: PhantomData<T>,
    pub sum: Vec<u8>,
    pub hash: u64,
    pub count: i64,
}

/// If a symbol can be successfully 'peeled' out of a CodedSymbol, it is returned in the PeelableResult.
///
/// This enum acts as a wrapper to keep track of if the symbol was local or remote.
///
/// It is very common that the symbol is not peelable, so this enum also has the NotPeelable variant.
#[derive(PartialEq, Eq, Clone, Debug)]
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
    pub fn new() -> Self {
        let sum = vec![0u8; T::BYTE_ARRAY_LENGTH];
        let hash = 0;
        let count = 0;
        CodedSymbol {
            _marker: PhantomData,
            sum,
            hash,
            count,
        }
    }

    /// apply() adds or removes a symbol from the CodedSymbol
    ///
    /// Adding a local, or removing a remote, symbol increases the count by 1
    ///
    /// Removing a local, or adding a remote, symbol decreases the count by 1
    pub fn apply(&mut self, s: &T, direction: Direction) {
        //It might be nice to split this into an 'add' and 'remove'
        assert_eq!(
            self.sum.len(),
            T::BYTE_ARRAY_LENGTH,
            "self.sum must have the length specified by T::BYTE_ARRAY_LENGTH."
        );
        let encoded_s = s.encode_to_bytes();

        assert_eq!(
            encoded_s.len(),
            T::BYTE_ARRAY_LENGTH,
            "encoded_s must have the length specified by T::BYTE_ARRAY_LENGTH."
        );

        // Should be able to update in place here
        self.sum = self
            .sum
            .iter()
            .zip(encoded_s.iter())
            .map(|(x, y)| x ^ y)
            .collect();

        self.hash ^= s.hash_();
        match direction {
            Direction::Add => self.count += 1,
            Direction::Remove => self.count -= 1,
        };
    }

    /// Used by the encoder to join two vectors of codedSymbols together produced from two distinct sets.
    /// The results are only valid if there were no duplicates between the original sets.
    pub fn combine(&self, b: &CodedSymbol<T>) -> CodedSymbol<T> {
        assert_eq!(
            self.sum.len(),
            T::BYTE_ARRAY_LENGTH,
            "self.sum must have the length specified by T::BYTE_ARRAY_LENGTH."
        );
        assert_eq!(
            b.sum.len(),
            T::BYTE_ARRAY_LENGTH,
            "encoded_s must have the length specified by T::BYTE_ARRAY_LENGTH."
        );

        let mut new_coded_symbol = self.clone();

        new_coded_symbol.hash ^= b.hash;
        new_coded_symbol.count += b.count;

        new_coded_symbol.sum = self
            .sum
            .iter()
            .zip(b.sum.iter())
            .map(|(x, y)| x ^ y)
            .collect();

        new_coded_symbol
    }

    /// Used by the encoder to 'subtract' a remote set of codedSymbols from a local set.
    pub fn collapse(&self, b: &CodedSymbol<T>) -> CodedSymbol<T> {
        assert_eq!(
            self.sum.len(),
            T::BYTE_ARRAY_LENGTH,
            "self.sum must have the length specified by T::BYTE_ARRAY_LENGTH."
        );
        assert_eq!(
            b.sum.len(),
            T::BYTE_ARRAY_LENGTH,
            "encoded_s must have the length specified by T::BYTE_ARRAY_LENGTH."
        );

        let mut new_coded_symbol = self.clone();

        // new_coded_symbol.symbol = new_coded_symbol.symbol.xor(&b.symbol);
        new_coded_symbol.hash ^= b.hash;
        new_coded_symbol.count -= b.count;

        new_coded_symbol.sum = self
            .sum
            .iter()
            .zip(b.sum.iter())
            .map(|(x, y)| x ^ y)
            .collect();

        new_coded_symbol
    }

    /// Checks if the CodedSymbol contains only one symbol and therefore can be peeled
    ///
    /// A count of 1 does not necessarily mean that the 'sum' field is the xor of only one encoded
    /// symbol. It could be the xor of two local and one remote symbols. This is why we also
    /// check the hash.
    pub fn is_peelable(&self) -> bool {
        if self.count == 1 || self.count == -1 {
            if self.hash == T::decode_from_bytes(&self.sum).hash_() {
                return true;
            }
        }
        return false;
    }

    /// Peel extracts a symbol from the CodedSymbol (if possible) and returns it in a PeelableResult
    /// A PeelableResult is used to keep track of if the symbol was local or remote (or was not
    /// able to be peeled).
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
    /// same as peel, but does not modify the CodedSymbol
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

    /// Checks if the CodedSymbol contains no symbols
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
    }
}
