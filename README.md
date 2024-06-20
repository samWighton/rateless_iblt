# Rateless IBLT

https://arxiv.org/html/2402.02668v2

The aim of this Rust Crate is to allow efficient set reconciliation over a network interface. This is acheived using Rateless Invertible Bloom Lookup Tables (Rateless IBLT).

This crate is based on the paper titled 'Practical Rateless Set Reconciliation' authored by Lei Yang, Yossi Gilad, Mohammad Alizadeh.

I intend to include additional functionality to help with cases where the sets are stored on disk. 

## General challenges for very large sets

By their definition, Sets can't have duplicates.
When storing a set in memory in rust, a hashset or BTreeSet can be used.
However, when the set is very large, the memory requirements can be prohibitive.

By definition, insertion when an element already exists is a no-op.
Enforcing this behaviour if the set is stored as an unordered list on disk, checking for a duplicate (before insertion) requires a full scan of the list.

Accompanying data structures, such as a regular bloom filter could reduce the need for a full scan.
If the entry is not in the bloom filter, it known to not yet be in the set, so we can insert/append it safely.
If the entry is in the bloom filter, it might be in the set, so we will need to do a full scan.

## Challenges for rapidly changing sets

Considering the use-case of keeping an insert-only set in sync across multiple servers.
It becomes practical to have three mechanisms for sharing data between servers.
1. A full set transfer, for cases when a new server is added or there are massive differences.
2. A streaming gossip mechanism. An insert on one server is broadcast to all other servers.
3. A repair mechanism, that is run periodically to ensure that all servers have the same set.

Assuming that Rateless IBLT is used for the repair mechanism.
Also assuming that the original insert time for each item is known and that the servers have a clock that is roughly in sync.

Because of the constant insertions to different servers, during busy periods, it is unlikely that the sets will be in sync.
This will result in wasteful use of the repair mechanism, as most differences will be resolved by the gossip mechanism.

To solve for this problem, when computing and sharing the Rateless IBLT, we should ignore items that were inserted within a certain time window.

For example, consider a system where 99.999% of items reach all servers within 10 seconds.
Every server could compute the Rateless IBLT on the minute, every minute for all items that were inserted more than 10 seconds ago.
Servers could share the coded symbols from the Rateless IBLT to a number of other servers. With this information, the servers could begin requesting missing items.

The repair mechanism would also handle cases of a network partition. Rateless IBLT would then be used to efficiently reconcile the differences.

## Interface presented by the crate

- Create a CodedSymbols structure from a iterable set
- Create a CodedSymbols structure from a network stream
- Create an empty CodedSymbols structure 
- Add an item to the CodedSymbols structure
- Remove an item from the CodedSymbols structure
- Combine two CodedSymbols structures (that were built from distinct sets)
- Collapse a local and remote CodedSymbols structure together
- Get a particular length (with optional offset) of CodedSymbols (to send over the network)
- Iterate to 'peel' off Symbols
- dry-run to check if we can peel to an empty set, this let's us know if we have received enough of a remote stream of CodedSymbols

