use crate::mapping;
use crate::symbol;

// constant for block size
// This could be custom for each server,
// Massive values reduce iterations over the set, but increase memory usage and are more likely to
// be generating CodedSymbols that are not used
//
// It might make sense to set a BLOCK_SIZE that is inversly proportional to size of the Symbol
pub const BLOCK_SIZE: u64 = 20;

// a function that takes a set that can be iterted over and an offset and returns a block of coded symbols

pub fn produce_block<T, I>(iterable: I, offset: u64) -> Vec<symbol::CodedSymbol<T>>
where
    I: IntoIterator<Item = T>,
    T: symbol::Symbol,
{
    let mut block = Vec::new();
    for _ in 0_u64..BLOCK_SIZE {
        block.push(symbol::CodedSymbol::new());
    }

    for item in iterable.into_iter() {
        // println!("item {:?}", item);
        let item_mapping = mapping::RandomMapping::new(&item);

        for i in item_mapping
            .take_while(|&x| x < offset + BLOCK_SIZE)
            .filter(|&x| x >= offset)
        {
            block[(i - offset) as usize].apply(
                &symbol::HashedSymbol::new(item.clone()),
                symbol::Direction::Add,
            );
            // println!("    item mapping {}", i);
        }
    }

    return block;
}

pub fn peel_one_symbol<T: symbol::Symbol>( block: &mut Vec<symbol::CodedSymbol<T>>) -> symbol::PeelableResult<T> {

    if block.is_empty() {
        return symbol::PeelableResult::NotPeelable;
    }
    let mut peelable_result = symbol::PeelableResult::NotPeelable;

    // we check if each codedSymbol can be peeled,
    // if it can, we exit the loop, remove it from the block and return the result
    for symbol in block.iter() {
        peelable_result = symbol.peel_peek();

        match peelable_result {
            symbol::PeelableResult::NotPeelable => { continue }
            _ => { 
                remove_symbol_from_block(block, peelable_result.clone());
                break 
            }
        }
    }

    peelable_result
}

pub fn remove_symbol_from_block<T: symbol::Symbol>(
    block: &mut Vec<symbol::CodedSymbol<T>>,
    symbol_result: symbol::PeelableResult<T>,
) {
    let direction;
    let symbol: T = match symbol_result {
        symbol::PeelableResult::Local(symbol) => {
            direction = symbol::Direction::Remove;
            symbol
        }
        symbol::PeelableResult::Remote(symbol) => {
            direction = symbol::Direction::Add;
            symbol
        }
        symbol::PeelableResult::NotPeelable => {
            panic!("Can't remove nothing from a block");
        }
    };

    let item_mapping = mapping::RandomMapping::new(&symbol);

    let block_len = block.len();

    for i in item_mapping.take_while(|&x| (x as usize) < block_len) {
        block[i as usize].apply(
            &symbol::HashedSymbol::new(symbol.clone()),
            direction.clone(),
        );
    }
}

// used to combine two blocks of coded symbols generated from two distinct sets
pub fn combine<T: symbol::Symbol>(
    block_a: &Vec<symbol::CodedSymbol<T>>,
    block_b: &Vec<symbol::CodedSymbol<T>>,
) -> Vec<symbol::CodedSymbol<T>> {
    let mut combined_block = Vec::new();

    for (a, b) in block_a.iter().zip(block_b.iter()) {
        combined_block.push(a.combine(b));
    }
    combined_block
}

// A collapsed block should effectively be the difference between two blocks
pub fn collapse<T: symbol::Symbol>(
    block_local: &Vec<symbol::CodedSymbol<T>>,
    block_remote: &Vec<symbol::CodedSymbol<T>>,
) -> Vec<symbol::CodedSymbol<T>> {
    let mut combined_block = Vec::new();

    for (coded_symbol_local, coded_symbol_remote) in block_local.iter().zip(block_remote.iter()) {
        combined_block.push(coded_symbol_local.collapse(coded_symbol_remote));
    }
    combined_block
}

pub fn is_empty<T: symbol::Symbol>( block: &Vec<symbol::CodedSymbol<T>>) -> bool {
    block.iter().all(|x| x.is_empty())
}

// pub fn invert_count<T: symbol::Symbol>(block: Vec<symbol::CodedSymbol<T>>) {
//     for mut coded_symbol in block {
//         coded_symbol.count = -coded_symbol.count;
//     }
// }
