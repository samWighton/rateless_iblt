use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;
use bincode;
use riblt;

// Example implementation of a struct that implements the Symbol trait
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SimpleSymbol {
    unique_id: u64,
    timestamp: u64,
}

impl riblt::Symbol for SimpleSymbol {
    const BYTE_ARRAY_LENGTH: usize = 16;
    // The Symbol trait only requires that the type can be encoded to bytes.
    // In this function we are doing so in a fairly manual way.
    // I would recommend using a serialization library like bincode.
    // You just need to know the size of the byte array that will be produced and then set BYTE_ARRAY_LENGTH to match.
    fn encode_to_bytes(&self) -> Vec<u8> {
        let mut buffer = vec![0u8; 16];
        buffer[0..8].copy_from_slice(&self.unique_id.to_le_bytes());
        buffer[8..16].copy_from_slice(&self.timestamp.to_le_bytes());
        buffer
    }
    fn decode_from_bytes(bytes: &Vec<u8>) -> Self {
        let unique_id = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let timestamp = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        SimpleSymbol {
            unique_id,
            timestamp,
        }
    }
    fn hash_(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.unique_id.hash(&mut hasher);
        hasher.finish()
    }
    // fn xor(&mut self, other: &Self) -> Self {
    //     Self {
    //         unique_id: self.unique_id ^ other.unique_id,
    //         timestamp: self.timestamp ^ other.timestamp,
    //     }
    // }
    // fn empty() -> Self {
    //     SimpleSymbol {
    //         unique_id: 0,
    //         timestamp: 0,
    //     }
    // }
}

fn main() {
    use std::collections::HashSet;

    // A set of symbols we have on our local server
    let local_items: HashSet<SimpleSymbol> = HashSet::from([
        SimpleSymbol { unique_id: 7, timestamp: 0},
        SimpleSymbol { unique_id: 15, timestamp: 0},
        SimpleSymbol { unique_id: 16, timestamp: 0},
        SimpleSymbol { unique_id: 17, timestamp: 0}, //local only
    ]);
    let mut managed_local_iblt = riblt::RatelessIBLT::new(local_items);

    // A set of symbols on a remote server
    let remote_items: HashSet<SimpleSymbol> = HashSet::from([
        SimpleSymbol { unique_id: 7, timestamp: 0},
        SimpleSymbol { unique_id: 15, timestamp: 0},
        SimpleSymbol { unique_id: 16, timestamp: 0},
        SimpleSymbol { unique_id: 18, timestamp: 0}, //remote only
    ]);
    let mut managed_remote_iblt = riblt::RatelessIBLT::new(remote_items);

    let mut local_copy_of_remote : riblt::UnmanagedRatelessIBLT<SimpleSymbol> = riblt::UnmanagedRatelessIBLT::new();
    for i in 0..20 {
        println!("Getting coded symbol {}", i);
        let one_coded_symbol = managed_remote_iblt.get_coded_symbol(i);
        let encoded_coded_symbol = bincode::serialize(&one_coded_symbol).unwrap();
        let decoded_coded_symbol : riblt::CodedSymbol<SimpleSymbol> = bincode::deserialize(&encoded_coded_symbol).unwrap();
        local_copy_of_remote.add_coded_symbol(&decoded_coded_symbol);

        let mut collapsed = managed_local_iblt.collapse(&local_copy_of_remote);
        let peeled_symbols = collapsed.peel_all_symbols();
        if collapsed.is_empty() {
            println!("Peeled all symbols");
            println!("{:?}", peeled_symbols);
            break;
        }
    }




    let mut test_items: HashSet<SimpleSymbol> = HashSet::new();
    for i in 0..10_000_000 {
        test_items.insert(SimpleSymbol {
            unique_id: i,
            timestamp: 0,
        });
    }
    let start = Instant::now();
    let mut test_riblt = riblt::RatelessIBLT::new(test_items);
    let coded_symbols_to_get = 10_000_000;
    let coded_symbol = test_riblt.get_coded_symbol(coded_symbols_to_get);
    let duration = start.elapsed();

    let encoded_coded_symbol = bincode::serialize(&coded_symbol).unwrap();
    let encoded_length = encoded_coded_symbol.len();
    println!("encoded CodedSymbol length {:?}", encoded_length);
    println!("encoding bandwidth {:?} Mb/s", coded_symbols_to_get as f64 * 8.0 * encoded_length as f64 / duration.as_secs_f64() / 1_000_000.0);

    println!("Time building codedSymbols is: {:?}", duration);

    // The time to produce 1,000,000 coded symbols (40 bytes each) from a set of 10,000,000 symbols is about 20 seconds
    // on my machine.
    //
    // This gives a 17 Mb/s encoding bandwidth.
    //
    // Encoding 10x more CodedSymbols takes about 34 seconds, which is a 98 Mb/s encoding bandwidth.
    //
    // This scales linearly with the number of symbols in the set.
    //
    // Next up is to test the decoding bandwidth of a collapsed RIBLT.
    //
    // overall this is looking quite positive, as we can encoded about as fast as we could get data
    // off a disk. And decoding should remain proportional to the differences in the sets.

    // let local_coded_symbol_block = riblt::produce_block(local_items, 0);
    // let remote_coded_symbol_block = riblt::produce_block(remote_items, 0);

    // let collapsed_coded_symbol_block =
    //     riblt::collapse(&local_coded_symbol_block, &remote_coded_symbol_block);
    // println!("{:?}", collapsed_coded_symbol_block);

    // let test_data = SimpleSymbol { unique_id: 17 };
    // let test_data_encoded = test_data.encode_to_bytes();
    // let test_data_decoded = SimpleSymbol::decode_from_bytes(&test_data_encoded);
    // println!("{:?}", test_data_decoded);


    // println!("{:?}", managed_local_iblt.coded_symbols);
    // managed_local_iblt.get_coded_symbol(1);
    // managed_local_iblt.get_coded_symbol(30);
    // managed_local_iblt.get_coded_symbol(13_000);
    // for cs in managed_local_iblt.coded_symbols.iter() {
    //     println!("{:?}", cs);
    // }

    // let get_coded_symbol = managed_local_iblt.get_coded_symbol(0);
    // let encoded_coded_symbol = bincode::serialize(&get_coded_symbol).unwrap();
    // println!("{:?}", encoded_coded_symbol);
    // println!("{:?}", managed_local_iblt.coded_symbols.len());
    // println!("{:?}", managed_local_iblt.coded_symbols);

    // Expected usage,
    // Local server asks remote server to begin sending a stream of coded symbols
    // Local server collapses against its own coded symbols to see if it can be peeled to zero
}
