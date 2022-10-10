pub type RandomState = ahash::RandomState;

pub type HashMap<K, V> = hashbrown::HashMap<K, V, RandomState>;
pub type HashSet<T> = hashbrown::HashSet<T, RandomState>;
