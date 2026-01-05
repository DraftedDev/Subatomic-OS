/// A fast hash map. Uses [FastHasher] as the hashing algorithm.
pub type FastMap<K, V> = hashbrown::hash_map::HashMap<K, V, FastHasher>;

/// A fast hash set. Uses [FastHasher] as the hashing algorithm.
pub type FastSet<T> = hashbrown::hash_set::HashSet<T, FastHasher>;

/// A fast hash map. Uses [rustc_hash::FxBuildHasher] as the hashing algorithm.
pub type FastHasher = rustc_hash::FxBuildHasher;

/// A fixed-size array with [Vec](alloc::vec::Vec)-like API.
pub type Array<T, const N: usize> = heapless::Vec<T, N>;

/// A stack-allocated string with [String](alloc::string::String)-like API.
pub type StackString<const N: usize> = heapless::String<N>;

pub use heapless::format as stack_format;
