use crate::mapping;
use crate::symbol;

/// Constant for block size. 
/// As it can be computationally expensive to iterate over the set, it makes sense to generate
/// a 'block' of coded symbols at a time.
///
/// Massive values reduce iterations over the set, but increase memory usage and are more likely to
/// be generating CodedSymbols that are not used
///
/// It might make sense to set a BLOCK_SIZE that is inversly proportional to size of the Symbol
pub const BLOCK_SIZE: usize = 1024;

/// There is a managed and unmanaged version of the RatelessIBLT
/// It is expected that the managed version will be used when we have access to the set
/// The managed version will generate coded symbols as needed (for efficiencey, it will generate a 'block' of coded symbols at a time)
/// The unmanaged version will be used whereever we don't have access to the set
pub struct RatelessIBLT<T, I>
where
    T: symbol::Symbol,
    I: IntoIterator<Item = T> + Clone,
{
    pub coded_symbols: Vec<symbol::CodedSymbol<T>>,
    set_iterator: I,
}

// It might be nice to 'peel' the symbols out as an iterator
// impl<T, I> Iterator for RatelessIBLT<T, I>
// where
//     T: symbol::Symbol,
//     I: IntoIterator<Item = T> + Clone,
// {
//     type Item = T;
// 
//     fn next(&mut self) -> Option<Self::Item> {
//         todo!();
//     }
// }

impl<T, I> RatelessIBLT<T, I>
where
    T: symbol::Symbol,
    I: IntoIterator<Item = T> + Clone,
{
    /// CodedSymbols are created as required, this method extends the codedSymbols to at least the provided index
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

    /// Returns the coded symbol at the provided index.
    ///
    /// It is expected that this will be called in a loop to stream the coded symbols to a remote server.
    ///
    /// If the index is greater than the current length of the coded symbols, we extend the coded symbols.
    pub fn get_coded_symbol(&mut self, index: usize) -> symbol::CodedSymbol<T> {
        if index >= self.coded_symbols.len() {
            self.extend_coded_symbols(index);
        }
        self.coded_symbols[index].clone()
    }

    /// Constructing a new RatelessIBLT requires a set of symbols that can be iterated over.
    /// The RatelessIBLT will generate coded symbols as needed. So this set may be iterated over multiple times.
    ///
    /// It is the responsibility of the calling code to create a new RatelessIBLT if the set changes.
    pub fn new(set_iterator: I) -> Self {
        let mut riblt = RatelessIBLT {
            coded_symbols: Vec::new(),
            set_iterator,
        };
        riblt.extend_coded_symbols(0);
        riblt
    }

    /// Join two vectors of codedSymbols together produced from two distinct sets.
    /// The results are only valid if there were no duplicates between the original sets.
    pub fn combine(&mut self, other: &RatelessIBLT<T, I>) -> UnmanagedRatelessIBLT<T> {
        // if the passed in RatelessIBLT has more coded symbols than self, we extend Self
        self.extend_coded_symbols(other.coded_symbols.len());
        combine(&self.coded_symbols, &other.coded_symbols)
    }

    /// Subtract a remote sequence of codedSymbols from a local sequence.
    pub fn collapse(&mut self, other: &UnmanagedRatelessIBLT<T>) -> UnmanagedRatelessIBLT<T> {
        // if the passed in RatelessIBLT has more coded symbols than self, we extend Self
        self.extend_coded_symbols(other.coded_symbols.len());
        collapse(&self.coded_symbols, &other.coded_symbols)
    }

    /// If possible, peel a single symbol from the RatelessIBLT
    pub fn peel_one_symbol(&mut self) -> symbol::PeelableResult<T> {
        peel_one_symbol(&mut self.coded_symbols)
    }


    /// Peel all symbols from the RatelessIBLT that we possibly can
    ///
    /// It is not expected that this would be called on a RatelessIBLT as we still have access to
    /// the set that we used to construct it.
    ///
    /// We expect to call this on an UnmanagedRatelessIBLT that was produced from collapsing a
    /// remote against our local
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

    /// returns true if there are no symbols
    /// If we can't peel any symbols, but it is not empty it means that we have symbols that
    /// can't be recovered
    /// We can't know if the RatelessIBLT is empty until we have iterated over the set
    pub fn is_empty(&mut self) -> bool {
        self.extend_coded_symbols(0); // This does nothing if we already have some coded symbols
        is_empty(&self.coded_symbols)
    }
}

/// The unmanaged version of the RatelessIBLT is used when we don't have access to the set.
/// It is also used when we want to combine or collapse two RatelessIBLTs.
///
/// In expected use, we will have a RatelessIBLT constructed from a locally available set of symbols.
/// We will use an UnmanagedRatelessIBLT to hold the coded symbols streamed from a remote server.
///
/// Collapsing the two will give us the symbols that were in the remote set but not in the local set.
/// We can use these to correct our local set.
///
/// It will also give us the symbols that were in the local set but not in the remote set.
/// We could send these to the remote server to correct their set.
pub struct UnmanagedRatelessIBLT<T>
where
    T: symbol::Symbol,
{
    pub coded_symbols: Vec<symbol::CodedSymbol<T>>,
}

// It might be nice to 'peel' the symbols out as an iterator
// impl<T> Iterator for UnmanagedRatelessIBLT<T>
// where
//     T: symbol::Symbol,
// {
//     type Item = T;
// 
//     fn next(&mut self) -> Option<Self::Item> {
//         //TODO
//         None
//     }
// }
impl<T> UnmanagedRatelessIBLT<T>
where
    T: symbol::Symbol,
{
    pub fn new() -> Self {
        return UnmanagedRatelessIBLT {
            coded_symbols: Vec::new(),
        };
    }

    /// Join two vectors of codedSymbols together produced from two distinct sets.
    /// The results are only valid if there were no duplicates between the original sets.
    pub fn combine(&self, other: &UnmanagedRatelessIBLT<T>) -> UnmanagedRatelessIBLT<T> {
        combine(&self.coded_symbols, &other.coded_symbols)
    }
    /// Subtract a remote sequence of codedSymbols from a local sequence.
    pub fn collapse(&self, other: &UnmanagedRatelessIBLT<T>) -> UnmanagedRatelessIBLT<T> {
        collapse(&self.coded_symbols, &other.coded_symbols)
    }
    /// If possible, peel a single symbol from the RatelessIBLT
    pub fn peel_one_symbol(&mut self) -> symbol::PeelableResult<T> {
        peel_one_symbol(&mut self.coded_symbols)
    }
    /// Peel all symbols from the RatelessIBLT that we possibly can
    /// Call the is_empty method to check if there are any symbols left
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
    /// Add a coded symbol
    /// The expected use is that a remote server is streaming us codedSymbols and we are adding them to our local copy.
    pub fn add_coded_symbol(&mut self, other: &symbol::CodedSymbol<T>) {
        self.coded_symbols.push(other.clone());
    }

    /// returns true if there are no symbols
    /// If we can't peel any symbols, but it is not empty it means that we have symbols that
    /// can't be recovered
    /// If there are no CodedSymbols, this will return true
    pub fn is_empty(&self) -> bool {
        //It might be good to panic if there are no coded symbols
        is_empty(&self.coded_symbols)
    }
}

// a function that takes a set that can be iterted over and an offset and returns a block of coded symbols

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
) -> UnmanagedRatelessIBLT<T> {
    let mut combined_block = Vec::new();

    for (a, b) in block_a.iter().zip(block_b.iter()) {
        combined_block.push(a.combine(b));
    }
    UnmanagedRatelessIBLT {
        coded_symbols: combined_block,
    }
}

// A collapsed block should effectively contain the difference between two blocks
pub fn collapse<T: symbol::Symbol>(
    block_local: &Vec<symbol::CodedSymbol<T>>,
    block_remote: &Vec<symbol::CodedSymbol<T>>,
) -> UnmanagedRatelessIBLT<T> {
    let mut combined_block = Vec::new();

    for (coded_symbol_local, coded_symbol_remote) in block_local.iter().zip(block_remote.iter()) {
        combined_block.push(coded_symbol_local.collapse(coded_symbol_remote));
    }
    UnmanagedRatelessIBLT {
        coded_symbols: combined_block,
    }
}

pub fn is_empty<T: symbol::Symbol>(block: &Vec<symbol::CodedSymbol<T>>) -> bool {
    block.iter().all(|x| x.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::SimpleSymbol;

    #[test]
    fn test_collapsing() {
        use std::collections::HashSet;

        let items_local: HashSet<SimpleSymbol> = HashSet::from([
            SimpleSymbol { value: 7 },
            SimpleSymbol { value: 15 },
            SimpleSymbol { value: 16 },
            SimpleSymbol { value: 2 },
        ]);

        let items_remote: HashSet<SimpleSymbol> = HashSet::from([
            SimpleSymbol { value: 7 },
            SimpleSymbol { value: 15 },
            SimpleSymbol { value: 16 },
            SimpleSymbol { value: 1 },
        ]);

        let mut iblt_local = RatelessIBLT::new(items_local.clone());
        iblt_local.extend_coded_symbols(0);
        let mut iblt_remote = RatelessIBLT::new(items_remote.clone());
        iblt_remote.extend_coded_symbols(0);

        let local_only: HashSet<SimpleSymbol> =
            items_local.difference(&items_remote).cloned().collect();
        let remote_only: HashSet<SimpleSymbol> =
            items_remote.difference(&items_local).cloned().collect();


        let iblt_remote_unmanaged : UnmanagedRatelessIBLT<SimpleSymbol> = UnmanagedRatelessIBLT {
            coded_symbols: iblt_remote.coded_symbols.clone(),
        };

        let mut collapsed_local = iblt_local.collapse(&iblt_remote_unmanaged);

        let mut peeled_set_local = HashSet::new();
        let mut peeled_set_remote = HashSet::new();

        for s in collapsed_local.peel_all_symbols() {
            match s {
                symbol::PeelableResult::Local(symbol) => {
                    peeled_set_local.insert(symbol.clone());
                }
                symbol::PeelableResult::Remote(symbol) => {
                    peeled_set_remote.insert(symbol.clone());
                }
                symbol::PeelableResult::NotPeelable => panic!("Not expecting this case"),
            }
        }
        assert_eq!(local_only, peeled_set_local);
        assert_eq!(remote_only, peeled_set_remote);
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

        let mut iblt = RatelessIBLT::new(items.clone());
        iblt.extend_coded_symbols(0);

        let mut peeled_set = HashSet::new();

        for s in iblt.peel_all_symbols() {
            match s {
                symbol::PeelableResult::Local(symbol) => {
                    peeled_set.insert(symbol.clone());
                }
                symbol::PeelableResult::Remote(_) => panic!("Not expecting remote symbol"),
                symbol::PeelableResult::NotPeelable => panic!("Not expecting this case"),
            }
        }

        assert!(iblt.is_empty());
        assert_eq!(items, peeled_set);
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
