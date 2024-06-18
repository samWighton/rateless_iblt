// a symbol is a item in the set
//
// A CodedSymbol can be peeled when the count is 1 or -1 and the hash matches
//
// There are benefits to specifying that the Symbol is a fixed size such as a u64
// We probably want some function that gives the 'empty' of the type
// We could xor it with itself to get the empty, but that seems too fancy
use std::fmt::Debug;

pub trait Symbol: Clone + Debug {
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
    fn hash(&self) -> u64;
}

// HashedSymbol is the bundle of a symbol and its hash.
#[derive(Clone, Debug)]
pub struct HashedSymbol<T: Symbol> {
    pub symbol: T,
    pub hash: u64,
}

impl<T: Symbol> HashedSymbol<T> {
    pub fn new(symbol: T) -> Self {
        let hash = symbol.hash();
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
        CodedSymbol { symbol, hash, count}
    }
}

// pub const ADD: i64 = 1;
// pub const REMOVE: i64 = -1;

pub enum Direction {
    Add,
    Remove,
}

impl<T: Symbol> CodedSymbol<T> {
    pub fn apply(&mut self, s: &HashedSymbol<T>, direction: Direction) {
        self.symbol = self.symbol.xor(&s.symbol);
        self.hash ^= s.hash;
        match direction {
            Direction::Add => self.count += 1,
            Direction::Remove => self.count -= 1,
        };
    }
    pub fn is_peelable(&self) -> bool{
        if self.count == 1 || self.count == -1 {
            if self.hash == self.symbol.hash() {
                return true;
            }
        }
        return false;
    }

    pub fn peel(&mut self) -> Option<T> {
        if self.is_peelable() {
            let return_symbol = self.symbol.clone();
            *self = CodedSymbol::new();
            return Some(return_symbol);
        }
        return None;
    }
}
