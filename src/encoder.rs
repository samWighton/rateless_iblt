use crate::mapping;
use crate::symbol;

// constant for block size
// This could be custom for each server,
// Massive values reduce iterations over the set, but increase memory usage and are more likely to
// be generating CodedSymbols that are not used
//
// It might make sense to set a BLOCK_SIZE that is inversly proportional to size of the Symbol
pub const BLOCK_SIZE: usize = 20;

// There is a managed and unmanaged version of the RatelessIBLT
// It is expected that the managed version will be used when we have access to the set
// The managed version will generate coded symbols as needed (for efficiencey, it will generate a 'block' of coded symbols at a time)
// The unmanaged version will be used whereever we don't have access to the set

pub struct RatelessIBLT<T, I>
where
    T: symbol::Symbol,
    I: IntoIterator<Item = T> + Clone,
{
    pub coded_symbols: Vec<symbol::CodedSymbol<T>>,
    set_iterator: I,
}

impl<T, I> Iterator for RatelessIBLT<T, I>
where
    T: symbol::Symbol,
    I: IntoIterator<Item = T> + Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        //TODO
        None
    }
}
impl<T, I> RatelessIBLT<T, I>
where
    T: symbol::Symbol,
    I: IntoIterator<Item = T> + Clone,
{
    pub fn extend_coded_symbols(&mut self, index: usize) {
        // extend the coded symbols so that we can access the coded symbol at the provided index
        // if the index is within the current length of the coded_symbols, we do nothing
        let current_len = self.coded_symbols.len();
        if index < current_len {
            return;
        }

        // we should generate at minimum the BLOCK_SIZE number of coded symbols
        let extend_until = usize::max(index + 1, current_len + BLOCK_SIZE);

        for _ in current_len..extend_until {
            println!("Extending coded symbols");
            self.coded_symbols.push(symbol::CodedSymbol::new());
        }

        let cloned_set_iterator = self.set_iterator.clone();

        for item in cloned_set_iterator.into_iter() {
            let item_mapping = mapping::RandomMapping::new(&item);

            for i in item_mapping
                .take_while(|&x| x < extend_until)
                .filter(|&x| x >= current_len)
            {
                self.coded_symbols[i].apply(&item, symbol::Direction::Add);
            }
        }
    }
    pub fn get_coded_symbol(&mut self, index: usize) -> symbol::CodedSymbol<T> {
        if index >= self.coded_symbols.len() {
            self.extend_coded_symbols(index);
        }
        self.coded_symbols[index].clone()
    }
    pub fn new(set_iterator: I) -> Self {
        RatelessIBLT {
            coded_symbols: Vec::new(),
            set_iterator,
        }
    }
    pub fn combine(&mut self, other: &RatelessIBLT<T, I>) -> Vec<symbol::CodedSymbol<T>> {
        // if the passed in RatelessIBLT has more coded symbols than self, we extend Self
        self.extend_coded_symbols(other.coded_symbols.len());
        combine(&self.coded_symbols, &other.coded_symbols)
    }
    pub fn collapse(&mut self, other: &RatelessIBLT<T, I>) -> Vec<symbol::CodedSymbol<T>> {
        // if the passed in RatelessIBLT has more coded symbols than self, we extend Self
        self.extend_coded_symbols(other.coded_symbols.len());
        collapse(&self.coded_symbols, &other.coded_symbols)
    }
    pub fn peel_one_symbol(&mut self) -> symbol::PeelableResult<T> {
        peel_one_symbol(&mut self.coded_symbols)
    }
    pub fn peel_all_symbols(&mut self) -> Vec<symbol::PeelableResult<T>> {
        let mut peeled_symbols = Vec::new();
        loop {
            let peeled_symbol = self.peel_one_symbol();
            match peeled_symbol {
                symbol::PeelableResult::NotPeelable => {
                    break;
                }
                _ => {
                    peeled_symbols.push(peeled_symbol);
                }
            }
        }
        peeled_symbols
    }
    pub fn is_empty(&mut self) -> bool {
        //We can't know if the RatelessIBLT is empty until we have iterated over the set
        self.extend_coded_symbols(0); // This does nothing if we already have some coded symbols
        is_empty(&self.coded_symbols)
    }
}

pub struct UnmanagedRatelessIBLT<T>
where
    T: symbol::Symbol,
{
    coded_symbols: Vec<symbol::CodedSymbol<T>>,
}

impl<T> Iterator for UnmanagedRatelessIBLT<T>
where
    T: symbol::Symbol,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        //TODO
        None
    }
}
impl<T> UnmanagedRatelessIBLT<T>
where
    T: symbol::Symbol,
{
    //
    pub fn new() -> Self {
        return UnmanagedRatelessIBLT {
            coded_symbols: Vec::new(),
        };
    }
    pub fn combine(&self, other: &UnmanagedRatelessIBLT<T>) -> Vec<symbol::CodedSymbol<T>> {
        combine(&self.coded_symbols, &other.coded_symbols)
    }
    pub fn collapse(&self, other: &UnmanagedRatelessIBLT<T>) -> Vec<symbol::CodedSymbol<T>> {
        collapse(&self.coded_symbols, &other.coded_symbols)
    }
    pub fn peel_one_symbol(&mut self) -> symbol::PeelableResult<T> {
        peel_one_symbol(&mut self.coded_symbols)
    }
    pub fn peel_all_symbols(&mut self) -> Vec<symbol::PeelableResult<T>> {
        let mut peeled_symbols = Vec::new();
        loop {
            let peeled_symbol = self.peel_one_symbol();
            match peeled_symbol {
                symbol::PeelableResult::NotPeelable => {
                    break;
                }
                _ => {
                    peeled_symbols.push(peeled_symbol);
                }
            }
        }
        peeled_symbols
    }
    pub fn is_empty(&self) -> bool {
        is_empty(&self.coded_symbols)
    }
}

// a function that takes a set that can be iterted over and an offset and returns a block of coded symbols

pub fn produce_block<T, I>(iterable: I, offset: usize) -> Vec<symbol::CodedSymbol<T>>
where
    I: IntoIterator<Item = T>,
    T: symbol::Symbol,
{
    let mut block = Vec::new();
    for _ in 0..BLOCK_SIZE {
        block.push(symbol::CodedSymbol::new());
    }

    for item in iterable.into_iter() {
        // println!("item {:?}", item);
        let item_mapping = mapping::RandomMapping::new(&item);

        for i in item_mapping
            .take_while(|&x| x < offset + BLOCK_SIZE)
            .filter(|&x| x >= offset)
        {
            block[(i - offset) as usize].apply(&item, symbol::Direction::Add);
            // println!("    item mapping {}", i);
        }
    }

    return block;
}

pub fn peel_one_symbol<T: symbol::Symbol>(
    block: &mut Vec<symbol::CodedSymbol<T>>,
) -> symbol::PeelableResult<T> {
    if block.is_empty() {
        return symbol::PeelableResult::NotPeelable;
    }
    let mut peelable_result = symbol::PeelableResult::NotPeelable;

    // we check if each codedSymbol can be peeled,
    // if it can, we exit the loop, remove it from the block and return the result
    for symbol in block.iter() {
        peelable_result = symbol.peel_peek();

        match peelable_result {
            symbol::PeelableResult::NotPeelable => continue,
            _ => {
                remove_symbol_from_block(block, peelable_result.clone());
                break;
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
        block[i as usize].apply(&symbol, direction.clone());
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

pub fn is_empty<T: symbol::Symbol>(block: &Vec<symbol::CodedSymbol<T>>) -> bool {
    block.iter().all(|x| x.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::SimpleSymbol;

    #[test]
    fn test_encoder() {
        use std::collections::HashSet;

        let items: HashSet<SimpleSymbol> = HashSet::from([
            SimpleSymbol { value: 7 },
            SimpleSymbol { value: 15 },
            SimpleSymbol { value: 16 },
        ]);

        let items2: HashSet<SimpleSymbol> = HashSet::from([
            SimpleSymbol { value: 7 },
            SimpleSymbol { value: 15 },
            SimpleSymbol { value: 16 },
            SimpleSymbol { value: 1 },
        ]);

        let coded_symbol_block = produce_block(items.clone(), 0);
        let coded_symbol_block2 = produce_block(items2.clone(), 0);

        let collapsed_symbol_block = collapse(&coded_symbol_block, &coded_symbol_block2);

        for coded_symbol in coded_symbol_block {
            println!("original  {:?}", coded_symbol);
        }

        for coded_symbol in collapsed_symbol_block {
            println!("collapsed {:?}", coded_symbol);
        }
    }

    #[test]
    fn test_peeling() {
        use std::collections::HashSet;

        let items: HashSet<SimpleSymbol> = HashSet::from([
            SimpleSymbol { value: 7 },
            SimpleSymbol { value: 15 },
            SimpleSymbol { value: 16 },
        ]);

        // let items2: HashSet<SimpleSymbol> = HashSet::from([
        //     SimpleSymbol { value: 7 },
        //     SimpleSymbol { value: 15 },
        //     SimpleSymbol { value: 16 },
        //     SimpleSymbol { value: 1 },
        // ]);

        let mut coded_symbol_block = produce_block(items.clone(), 0);

        loop {
            let peeled_symbol = peel_one_symbol(&mut coded_symbol_block);
            match peeled_symbol {
                symbol::PeelableResult::Local(symbol) => {
                    println!("Peeled Local Symbol: {:?}", symbol);
                }
                symbol::PeelableResult::Remote(symbol) => {
                    println!("Peeled Remote Symbol: {:?}", symbol);
                }
                symbol::PeelableResult::NotPeelable => {
                    println!("No symbol to peel");
                    break;
                }
            }
        }

        assert!(is_empty(&coded_symbol_block));

        // assert!(false);
    }

    #[test]
    fn test_union() {
        use std::collections::HashSet;
        let common_items: HashSet<u64> = HashSet::from([1, 2, 3]);
        let a_only_items: HashSet<u64> = HashSet::from([4]);
        let b_only_items: HashSet<u64> = HashSet::from([5, 6]);

        let a: HashSet<u64> = common_items.union(&a_only_items).cloned().collect();
        let b: HashSet<u64> = common_items.union(&b_only_items).cloned().collect();

        let expected_difference_set: HashSet<u64> =
            a_only_items.union(&b_only_items).cloned().collect();
        let computed_difference_set: HashSet<u64> = a.symmetric_difference(&b).cloned().collect();

        assert_eq!(expected_difference_set, computed_difference_set);

        let mut items_missing_from_a: HashSet<u64> = HashSet::new();

        for item in &computed_difference_set {
            if !a.contains(item) {
                items_missing_from_a.insert(*item);
            }
        }
        assert_eq!(items_missing_from_a, b_only_items);

        // let elements: Vec<u64> = vec![1, 2, 3, 4, 5];
        // let result = elements.iter().copied().fold(0, |acc, x| acc ^ x);
        // println!("The XOR of all elements is: {}", result);
    }
}
