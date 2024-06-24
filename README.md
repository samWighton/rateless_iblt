# Rateless Invertible Bloom Lookup Table (RIBLT).

The aim of this Rust Crate is to allow efficient set reconciliation over a network interface. This is achieved using Rateless Invertible Bloom Lookup Tables (RIBLT).

This crate is based on the paper titled 'Practical Rateless Set Reconciliation' authored by Lei Yang, Yossi Gilad, Mohammad Alizadeh.

https://arxiv.org/html/2402.02668v2

This crate does not require a particular 'set' implementation, it only requires that the set is iterable. 
This allows the user to use any set implementation that is appropriate for their use-case, including a set read from disk.

Please note that this crate does not look for duplicates in the set. Duplicate items cannot be peeled out of the RIBLT.

## Glossary

- Symbol: An item in the set
- CodedSymbol: An element of the RIBLT.
- Peel: The process of removing a symbol from the RIBLT.

## Overview of what this crate gives you

### RatelessIBLT

A struct that is created by passing in an iterable set of symbols.

It will create the RIBLT codedSymbols as needed.

See the RatelessIBLT struct for more information.

### UnmanagedRatelessIBLT

Similar to the RatelessIBLT, but without the iterable set.

This is used when we don't have access to the set that created this RIBLT.

It is also used when we have 'combined' or 'collapsed' two RIBLTs together.

See the UnmanagedRatelessIBLT struct for more information.

## Hash collision probability

As described by the birthday paradox, the probability of a hash collision is 50% when the number of items in the set is equal to the square root of the possible outcomes. We are using 64-bit hashes, so we should be expecting hash collisions when we are around 4 billion items.


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

## Future work

### Async and multi-threading

There is currently no use of async or multi-threading in this crate. I will test the performance gains in the future.

This is considered a lower priority, as I am anticipating this will be used on a server that is performing other tasks.

## Notes

Currently it is the responsibility of the calling code to recreate the struct when the set changes.



