use riblt;

use std::hash::{DefaultHasher, Hash, Hasher};

// Example implementation of a struct that implements the Symbol trait
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SimpleSymbol {
    value: u64,
}

impl riblt::Symbol for SimpleSymbol {
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

fn main() {

    use std::collections::HashSet;

    let local_items: HashSet<SimpleSymbol> = HashSet::from([
        SimpleSymbol { value: 7 },
        SimpleSymbol { value: 15 },
        SimpleSymbol { value: 16 },
    ]);
    let remote_items: HashSet<SimpleSymbol> = HashSet::from([
        SimpleSymbol { value: 7 },
        SimpleSymbol { value: 15 },
        SimpleSymbol { value: 16 },
        SimpleSymbol { value: 17 },
    ]);

    let local_coded_symbol_block = riblt::produce_block(local_items, 0);
    let remote_coded_symbol_block = riblt::produce_block(remote_items, 0);

    let collapsed_coded_symbol_block = riblt::collapse(&local_coded_symbol_block, &remote_coded_symbol_block);
    collapsed_coded_symbol_block
    println!("{:?}", local_coded_symbol_block);
}
