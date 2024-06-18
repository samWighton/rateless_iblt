use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct SymbolMapping {
    source_idx: usize,
    coded_idx: usize,
}

struct MappingHeap(Vec<SymbolMapping>);

impl MappingHeap {
    fn new() -> Self {
        MappingHeap(Vec::new())
    }

    fn fix_head(&mut self) {
        let mut curr = 0;
        loop {
            let left_child = 2 * curr + 1;
            if left_child >= self.0.len() {
                break;
            }
            let right_child = left_child + 1;
            let smallest_child = if right_child < self.0.len() && self.0[right_child].coded_idx < self.0[left_child].coded_idx {
                right_child
            } else {
                left_child
            };
            if self.0[curr].coded_idx <= self.0[smallest_child].coded_idx {
                break;
            }
            self.0.swap(curr, smallest_child);
            curr = smallest_child;
        }
    }

    fn fix_tail(&mut self) {
        let mut curr = self.0.len() - 1;
        while curr > 0 {
            let parent = (curr - 1) / 2;
            if self.0[parent].coded_idx <= self.0[curr].coded_idx {
                break;
            }
            self.0.swap(parent, curr);
            curr = parent;
        }
    }

    fn push(&mut self, mapping: SymbolMapping) {
        self.0.push(mapping);
        self.fix_tail();
    }

    fn pop(&mut self) -> Option<SymbolMapping> {
        self.0.pop().map(|last| {
            if !self.0.is_empty() {
                let first = self.0.swap_remove(0);
                self.fix_head();
                first
            } else {
                last
            }
        })
    }
}

pub struct HashedSymbol<T> {
    value: T,
    hash: u64,
}

pub struct RandomMapping {
    hash: u64,
    last_idx: u64,
}

impl RandomMapping {
    fn next_index(&mut self) -> u64 {
        // Placeholder for actual implementation
        self.last_idx + 1
    }
}

pub struct CodedSymbol<T> {
    // Placeholder for actual fields
    _phantom: std::marker::PhantomData<T>,
}

impl<T> CodedSymbol<T> {
    fn apply(self, _symbol: &HashedSymbol<T>, _direction: i64) -> Self {
        // Placeholder for actual implementation
        self
    }
}

pub trait Symbol {
    fn hash(&self) -> u64;
}

pub struct CodingWindow<T: Symbol> {
    symbols: Vec<HashedSymbol<T>>,
    mappings: Vec<RandomMapping>,
    queue: MappingHeap,
    next_idx: usize,
}

impl<T: Symbol> CodingWindow<T> {
    pub fn new() -> Self {
        CodingWindow {
            symbols: Vec::new(),
            mappings: Vec::new(),
            queue: MappingHeap::new(),
            next_idx: 0,
        }
    }

    pub fn add_symbol(&mut self, t: T) {
        let hashed_symbol = HashedSymbol { value: t, hash: t.hash() };
        self.add_hashed_symbol(hashed_symbol);
    }

    pub fn add_hashed_symbol(&mut self, t: HashedSymbol<T>) {
        self.add_hashed_symbol_with_mapping(t, RandomMapping { hash: t.hash, last_idx: 0 });
    }

    pub fn add_hashed_symbol_with_mapping(&mut self, t: HashedSymbol<T>, m: RandomMapping) {
        self.symbols.push(t);
        self.mappings.push(m);
        self.queue.push(SymbolMapping { source_idx: self.symbols.len() - 1, coded_idx: m.last_idx as usize });
    }

    pub fn apply_window(&mut self, mut cw: CodedSymbol<T>, direction: i64) -> CodedSymbol<T> {
        while let Some(mut top) = self.queue.pop() {
            if top.coded_idx != self.next_idx {
                self.queue.push(top);
                break;
            }
            cw = cw.apply(&self.symbols[top.source_idx], direction);
            top.coded_idx = self.mappings[top.source_idx].next_index() as usize;
            self.queue.push(top);
        }
        self.next_idx += 1;
        cw
    }

    pub fn reset(&mut self) {
        self.symbols.clear();
        self.mappings.clear();
        self.queue.0.clear();
        self.next_idx = 0;
    }
}

pub struct Encoder<T: Symbol>(CodingWindow<T>);

impl<T: Symbol> Encoder<T> {
    pub fn new() -> Self {
        Encoder(CodingWindow::new())
    }

    pub fn add_symbol(&mut self, s: T) {
        self.0.add_symbol(s);
    }

    pub fn add_hashed_symbol(&mut self, s: HashedSymbol<T>) {
        self.0.add_hashed_symbol(s);
    }

    pub fn produce_next_coded_symbol(&mut self) -> CodedSymbol<T> {
        self.0.apply_window(CodedSymbol { _phantom: std::marker::PhantomData }, 1)
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }
}










