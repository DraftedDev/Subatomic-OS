/// A fast hash map. Uses `FxHash` as the hashing algorithm.
pub type FastMap<K, V> = hashbrown::hash_map::HashMap<K, V, rustc_hash::FxBuildHasher>;

/// A fast hash set. Uses `FxHash` as the hashing algorithm.
pub type FastSet<T> = hashbrown::hash_set::HashSet<T, rustc_hash::FxBuildHasher>;

/// A fixed-size array with [Vec](alloc::vec::Vec)-like API.
pub type Array<T, const N: usize> = heapless::Vec<T, N>;

/// A stack-allocated string with [String](alloc::string::String)-like API.
pub type StackString<const N: usize> = heapless::String<N>;

pub use heapless::format as stack_format;
