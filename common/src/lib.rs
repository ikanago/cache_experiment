use std::{borrow::Borrow, hash::Hash, rc::Rc};

/// Interface for cache.
pub trait Cache<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    /// Insert a new key-value pair.
    fn insert(&mut self, key: K, value: V);

    /// Get a clone of value corresponding to `key`.
    fn get<Q>(&mut self, key: &Q) -> Option<V>
    where
        Rc<K>: Borrow<Q>,
        Q: Eq + Hash + ?Sized;
}
