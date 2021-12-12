use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

pub(crate) type NodeRef<K, V> = Rc<RefCell<Node<K, V>>>;

pub(crate) struct Node<K, V> {
    pub(crate) next: Option<NodeRef<K, V>>,
    pub(crate) key: K,
    pub(crate) value: V,
}

impl<K, V> Node<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self {
            next: None,
            key,
            value,
        }
    }
}

pub struct SyncNaiveLru<K, V> {
    map: HashMap<K, NodeRef<K, V>>,
    pub(crate) head: Option<NodeRef<K, V>>,
    tail: Option<NodeRef<K, V>>,
}

impl<K, V> SyncNaiveLru<K, V>
where
    K: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            head: None,
            tail: None,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let node = Rc::new(RefCell::new(Node::new(key.clone(), value)));
        self.map.insert(key, Rc::clone(&node));
        self.attach(node);
    }

    /// Attach `node` to the head of linked list.
    fn attach(&mut self, node: NodeRef<K, V>) {
        if self.head.is_some() {
            (*node.borrow_mut()).next = Some(Rc::clone(&self.head.as_ref().unwrap()));
        } else {
            self.tail = Some(Rc::clone(&node));
        }
        // (*self.head.as_ref().unwrap().borrow_mut()).next = Some(Rc::clone(&node));
        self.head = Some(Rc::clone(&node));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterate() {
        let mut lru = SyncNaiveLru::new();
        vec![(1, 2), (3, 4), (5, 6)]
            .iter()
            .for_each(|kv| lru.insert(kv.0, kv.1));

        let tail = &lru.tail.as_ref().unwrap();
        assert_eq!((tail.borrow().key, tail.borrow().value), (1, 2));
        assert_eq!(
            lru.into_iter().collect::<Vec<_>>(),
            vec![(5, 6), (3, 4), (1, 2),]
        );
    }
}
