use riblt;
use riblt::Symbol;

use bincode;
use std::hash::{DefaultHasher, Hash, Hasher};

// Example implementation of a struct that implements the Symbol trait
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SimpleSymbol {
    unique_id: u64,
    // timestamp: u64,
}

impl riblt::Symbol for SimpleSymbol {
    // const BYTE_ARRAY_LENGTH: usize = 10;
    fn xor(&mut self, other: &Self) -> Self {
        Self {
            unique_id: self.unique_id ^ other.unique_id,
            // timestamp: self.timestamp ^ other.timestamp,
        }
    }
    fn hash_(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.unique_id.hash(&mut hasher);
        hasher.finish()
    }
    fn empty() -> Self {
        SimpleSymbol { unique_id: 0 }
    }

    // fn encode_to_bytes(&self) -> Vec<u8> {
    //     let mut buffer = vec![0u8; SimpleSymbol::BYTE_ARRAY_LENGTH];
    //     // buffer.extend_from_slice(&self.unique_id.to_le_bytes());
    //     buffer[0..8].copy_from_slice(&self.unique_id.to_le_bytes());

    //     // buffer[0..8].copy_from_slice(&self.unique_id.to_le_bytes());
    //     // buffer[2..4].copy_from_slice(&self.field2.to_le_bytes());

    //     buffer
    // }

    // fn decode_from_bytes(buffer: &Vec<u8>) -> Self {
    //     let unique_id = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
    //     // let field2 = u16::from_le_bytes(buffer[2..4].try_into().unwrap());

    //     SimpleSymbol { unique_id }
    // }
}

fn main() {
    use std::collections::HashSet;

    let mut local_items: HashSet<SimpleSymbol> = HashSet::from([
        // SimpleSymbol { unique_id: 7 },
        // SimpleSymbol { unique_id: 15 },
        // SimpleSymbol { unique_id: 16 },
    ]);

    for i in 0..10_000 {
        local_items.insert(SimpleSymbol { unique_id: i });
    }


    let remote_items: HashSet<SimpleSymbol> = HashSet::from([
        SimpleSymbol { unique_id: 7 },
        SimpleSymbol { unique_id: 15 },
        SimpleSymbol { unique_id: 16 },
        SimpleSymbol { unique_id: 17 },
    ]);

    // let local_coded_symbol_block = riblt::produce_block(local_items, 0);
    // let remote_coded_symbol_block = riblt::produce_block(remote_items, 0);

    // let collapsed_coded_symbol_block =
    //     riblt::collapse(&local_coded_symbol_block, &remote_coded_symbol_block);
    // println!("{:?}", collapsed_coded_symbol_block);

    // let test_data = SimpleSymbol { unique_id: 17 };
    // let test_data_encoded = test_data.encode_to_bytes();
    // let test_data_decoded = SimpleSymbol::decode_from_bytes(&test_data_encoded);
    // println!("{:?}", test_data_decoded);

    let mut managed_local_iblt = riblt::RatelessIBLT::new(local_items);
    let mut managed_remote_iblt = riblt::RatelessIBLT::new(remote_items);

    // println!("{:?}", managed_local_iblt.coded_symbols);
    managed_local_iblt.get_coded_symbol(1);
    managed_local_iblt.get_coded_symbol(30);
    managed_local_iblt.get_coded_symbol(13_000);
    for cs in managed_local_iblt.coded_symbols.iter() {
        println!("{:?}", cs);
    }
    println!("{:?}", managed_local_iblt.coded_symbols.len());
    // println!("{:?}", managed_local_iblt.coded_symbols);
}

