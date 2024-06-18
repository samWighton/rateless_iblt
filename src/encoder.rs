use crate::symbol;

// constant for block size
// This could be custom for each server,
// Massive values reduce iterations over the set, but increase memory usage and are more likely to
// be generating CodedSymbols that are not used
pub const BLOCK_SIZE: usize = 1024;

// a function that takes a set that can be iterted over and an offset and returns a block of coded symbols

// pub fn encode_block<T, I>(iterable: I) -> Vec<symbol::CodedSymbol<T>>
// where
// I: IntoIterator<Item = T>,
// {
//     iterable.into_iter().nth(9);
//     return symbol::CodedSymbol::new();
// 
// }
