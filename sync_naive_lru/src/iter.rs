use std::{hash::Hash, rc::Rc};

use crate::lru::{NodeRef, SyncNaiveLru};

pub struct IntoIter<K, V> {
    current: Option<NodeRef<K, V>>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            None => None,
            Some(current) => {
                let item = (
                    current.as_ref().borrow().key.clone(),
                    current.as_ref().borrow().value.clone(),
                );
                self.current = match &current.borrow().next {
                    Some(next) => Some(Rc::clone(&next)),
                    None => None,
                };
                Some(item)
            }
        }
    }
}

impl<K, V> SyncNaiveLru<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn into_iter(self) -> IntoIter<K, V> {
        IntoIter { current: self.head }
    }
}
