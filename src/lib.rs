mod encoder;
mod mapping;
mod symbol;

//functionality that we are going to need:
//- Function that takes an iterable set as input, and produces a large block of coded symbols
//- A higher level function that repeatedly calls the above function to produce a infinite stream
//  of coded symbols
//- Function that takes two lengths of coded symbols collapses them to get the differences and peels
//  out the symbols
//

// Re-export items from your modules
pub use encoder::produce_block;
pub use symbol::Symbol;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::hash::{DefaultHasher, Hash, Hasher};

    // Example implementation of a struct that implements the Symbol trait
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct SimpleSymbol {
        value: u64,
    }

    impl symbol::Symbol for SimpleSymbol {
        fn xor(&mut self, other: &Self) -> Self {
            Self {
                value: self.value ^ other.value,
            }
        }
        fn hash_(&self) -> u64 {
            let mut hasher = DefaultHasher::new();
            self.value.hash(&mut hasher);
            hasher.finish()
        }
        fn empty() -> Self {
            SimpleSymbol { value: 0 }
        }
    }

    #[test]
    fn test_mapping() {
        let rm = mapping::RandomMapping::new(&SimpleSymbol { value: 1 });

        for index in rm.take(10) {
            println!("{}", index);
        }
        let rm = mapping::RandomMapping::new(&SimpleSymbol { value: 2 });
        for index in rm.take(10) {
            println!("{}", index);
        }

        let rm = mapping::RandomMapping::new(&SimpleSymbol { value: 2 });

        //combining take_while and filter can give us the indexes that land in a range
        //helpful if we are computing the coded symbols in a block
        let below_100: Vec<u64> = rm.take_while(|&x| x <= 100).filter(|&x| x > 30).collect();
        println!("{:?}", below_100);
        // assert!(false);
    }

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

        let coded_symbol_block = encoder::produce_block(items.clone(), 0);
        let coded_symbol_block2 = encoder::produce_block(items2.clone(), 0);

        let collapsed_symbol_block = encoder::collapse(&coded_symbol_block, &coded_symbol_block2);

        for coded_symbol in coded_symbol_block {
            println!("original  {:?}", coded_symbol);
        }

        for coded_symbol in collapsed_symbol_block {
            println!("collapsed {:?}", coded_symbol);
        }

        // encoder::produce_block(, 0);
        // assert!(false);
    }

    #[test]
    fn test_function_0() {
        let common_items: HashSet<u64> = HashSet::from([1, 2, 3]);
        let a_only_items: HashSet<u64> = HashSet::from([4]);
        let b_only_items: HashSet<u64> = HashSet::from([5, 6]);

        let a: HashSet<u64> = common_items.union(&a_only_items).cloned().collect();
        let b: HashSet<u64> = common_items.union(&b_only_items).cloned().collect();

        let expected_difference_set: HashSet<u64> =
            a_only_items.union(&b_only_items).cloned().collect();
        let computed_difference_set: HashSet<u64> = a.symmetric_difference(&b).cloned().collect();

        assert_eq!(expected_difference_set, computed_difference_set);

        // Rateless IBLT will give us the equivalent of the symmetric_difference.
        // we actually want a list of the items we don't have only so we can request from another
        // server.
        //
        // We can expect a to be large and the symmetric_difference set to be small in most cases.
        // We should take care in how we aproach this if there is a cost in determining membership
        // to 'a'.
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

    #[test]
    fn test_symbol() {
        use symbol::Symbol;

        let symbol1 = SimpleSymbol { value: 42 };
        let symbol2 = SimpleSymbol { value: 100 };

        let hash_symbol1 = symbol::HashedSymbol {
            symbol: symbol1.clone(),
            hash: symbol1.hash_(),
        };
        let hash_symbol2 = symbol::HashedSymbol {
            symbol: symbol2.clone(),
            hash: symbol2.hash_(),
        };
        let mut coded_symbol = symbol::CodedSymbol::new();

        println!("0 is peelable {}", coded_symbol.is_peelable());
        assert_eq!(coded_symbol.is_peelable(), false);

        coded_symbol.apply(&hash_symbol1, symbol::Direction::Add);
        println!("1 is peelable {}", coded_symbol.is_peelable());
        assert_eq!(coded_symbol.is_peelable(), true);

        coded_symbol.apply(&hash_symbol2, symbol::Direction::Add);
        println!("2 is peelable {}", coded_symbol.is_peelable());
        assert_eq!(coded_symbol.is_peelable(), false);

        coded_symbol.apply(&hash_symbol1, symbol::Direction::Remove);
        println!("3 is peelable {}", coded_symbol.is_peelable());
        assert_eq!(coded_symbol.is_peelable(), true);

        println!("CodedSymbol: {:?}", coded_symbol);

        let peeled_symbol = coded_symbol.peel();
        match peeled_symbol {
            symbol::PeelableResult::Local(symbol) => {
                println!("Peeled Local Symbol: {:?}", symbol);
                assert_eq!(symbol.value, hash_symbol2.symbol.value);
            }
            symbol::PeelableResult::Remote(symbol) => {
                println!("Peeled Remote Symbol: {:?}", symbol);
                assert_eq!(symbol.value, hash_symbol2.symbol.value);
            }
            symbol::PeelableResult::NotPeelable => {
                println!("No symbol to peel");
                assert!(false);
            }
        }

        // assert!(false);
    }
}
