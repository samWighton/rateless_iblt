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

pub trait Symbol: Clone + Debug {
    // const BYTE_ARRAY_LENGTH: usize;
    // fn encode_to_bytes(&self) -> [u8; Self::BYTE_ARRAY_LENGTH];
    // fn decode_from_bytes(bytes: &[u8; Self::BYTE_ARRAY_LENGTH]) -> Self;

    fn empty() -> Self;

    // XOR returns t ^ t2, where t is the method receiver. XOR is allowed to
    // modify the method receiver. Although the method is called XOR (because
    // the bitwise exclusive-or operation is a valid group operation for groups
    // of fixed-length bit strings), it can implement any operation that
    // satisfy the aforementioned properties.
    fn xor(&mut self, other: &Self) -> Self;

    // Hash returns the hash of the method receiver. It must not modify the
    // method receiver. It must not be homomorphic over the group operation.
    // That is, the probability that
    //   (a ^ b).hash() == a.hash() ^ b.hash()
    // must be negligible. Here, ^ is the group operation on the left-hand
    // side, and bitwise exclusive-or on the right side.
    fn hash_(&self) -> u64;
}

// HashedSymbol is the bundle of a symbol and its hash.
#[derive(Clone, Debug)]
pub struct HashedSymbol<T: Symbol> {
    pub symbol: T,
    pub hash: u64,
}

impl<T: Symbol> HashedSymbol<T> {
    pub fn new(symbol: T) -> Self {
        let hash = symbol.hash_();
        HashedSymbol { symbol, hash }
    }
}

// coded symbol produced by a Rateless IBLT encoder.
// I have made the decision here to not use HashedSymbol as a subfield of CodedSymbol
// as it makes the implementation of new() more aligned with how this struct is used.
#[derive(Clone, Debug)]
pub struct CodedSymbol<T: Symbol> {
    pub symbol: T,
    pub hash: u64,
    pub count: i64,
}

impl<T: Symbol> CodedSymbol<T> {
    pub fn new() -> Self {
        let symbol = T::empty();
        let hash = 0;
        let count = 0;
        CodedSymbol {
            symbol,
            hash,
            count,
        }
    }
}

// pub const ADD: i64 = 1;
// pub const REMOVE: i64 = -1;

// We shall set the 'count' to negative for IBLT with a remote origin
// by this convention, when peeling, codedSymbols with a count of 1 are present only locally
// and codedSymbols with a count of -1 were present only in the remote set

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
    pub fn apply(&mut self, s: &HashedSymbol<T>, direction: Direction) {
        self.symbol = self.symbol.xor(&s.symbol);
        self.hash ^= s.hash;
        match direction {
            Direction::Add => self.count += 1,
            Direction::Remove => self.count -= 1,
        };
    }

    //used by the encoder to join two vectors of codedSymbols together produced from two distinct sets
    //The results are only valid if there were no duplicates between the original sets
    pub fn combine(&self, b: &CodedSymbol<T>) -> CodedSymbol<T> {
        let mut new_coded_symbol = self.clone();

        new_coded_symbol.symbol = new_coded_symbol.symbol.xor(&b.symbol);
        new_coded_symbol.hash ^= b.hash;
        new_coded_symbol.count += b.count;

        new_coded_symbol
    }

    pub fn collapse(&self, b: &CodedSymbol<T>) -> CodedSymbol<T> {
        let mut new_coded_symbol = self.clone();

        new_coded_symbol.symbol = new_coded_symbol.symbol.xor(&b.symbol);
        new_coded_symbol.hash ^= b.hash;
        new_coded_symbol.count -= b.count;

        new_coded_symbol
    }

    // this does not modify the CodedSymbol
    pub fn is_peelable(&self) -> bool {
        if self.count == 1 || self.count == -1 {
            if self.hash == self.symbol.hash_() {
                return true;
            }
        }
        return false;
    }

    pub fn peel(&mut self) -> PeelableResult<T> {
        if self.is_peelable() {
            let return_result = if self.count == 1 {
                PeelableResult::Local(self.symbol.clone())
            } else {
                PeelableResult::Remote(self.symbol.clone())
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
                PeelableResult::Local(self.symbol.clone())
            } else {
                PeelableResult::Remote(self.symbol.clone())
            };

            return return_result;
        }
        PeelableResult::NotPeelable
    }
}
