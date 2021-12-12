use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    rc::{Rc, Weak},
};

pub(crate) type NodeRef<K, V> = Rc<RefCell<Node<K, V>>>;
pub(crate) type NodeWeakRef<K, V> = Weak<RefCell<Node<K, V>>>;

pub(crate) struct Node<K, V> {
    pub(crate) next: Option<NodeRef<K, V>>,
    pub(crate) prev: Option<NodeWeakRef<K, V>>,
    pub(crate) key: K,
    pub(crate) value: V,
}

impl<K, V> Node<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self {
            next: None,
            prev: None,
            key,
            value,
        }
    }
}

pub struct SyncNaiveLru<K, V> {
    map: HashMap<K, NodeRef<K, V>>,
    head: Option<NodeRef<K, V>>,
    pub(crate) tail: Option<NodeRef<K, V>>,
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
            (*node.borrow_mut()).prev = Some(Rc::downgrade(self.head.as_ref().unwrap()));
            (*self.head.as_ref().unwrap().borrow_mut()).next = Some(Rc::clone(&node));
        } else {
            self.tail = Some(Rc::clone(&node));
        }
        self.head = Some(Rc::clone(&node));
    }

    fn detach(&mut self, node: NodeRef<K, V>) {
        match node.as_ref().borrow().prev.as_ref() {
            Some(prev) => match Weak::upgrade(prev) {
                Some(prev) => {
                    prev.borrow_mut().next = node.as_ref().borrow().next.clone();
                }
                None => panic!("previous is not None"),
            },
            None => {
                // `node` is reference to tail element.
                self.tail = node.as_ref().borrow().next.clone();
            }
        }

        match node.as_ref().borrow().next.as_ref() {
            Some(next) => {
                next.borrow_mut().prev = node.as_ref().borrow().prev.clone();
            }
            None => {
                // `node` is reference to head element.
                self.head = match node.as_ref().borrow().prev.as_ref() {
                    Some(prev) => Weak::upgrade(prev),
                    None => None,
                };
            }
        }
    }
}

impl<K: Hash + Eq + Clone, V> Default for SyncNaiveLru<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_insert() {
        let mut lru = SyncNaiveLru::new();
        let expected = vec![(1, 2), (3, 4), (5, 6)];
        expected.iter().for_each(|kv| lru.insert(kv.0, kv.1));

        let tail = lru.tail.as_ref().unwrap();
        assert_eq!(
            (tail.as_ref().borrow().key, tail.as_ref().borrow().value),
            (1, 2)
        );
        assert_eq!(lru.into_iter().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn detach_head() {
        let mut lru = SyncNaiveLru::new();
        vec![(1, 2), (3, 4), (5, 6)]
            .iter()
            .for_each(|kv| lru.insert(kv.0, kv.1));
        let node = Rc::clone(lru.map.get(&5).unwrap());
        lru.detach(node);
        assert_eq!(lru.into_iter().collect::<Vec<_>>(), vec![(1, 2), (3, 4)]);
    }

    #[test]
    fn detach_middle() {
        let mut lru = SyncNaiveLru::new();
        vec![(1, 2), (3, 4), (5, 6)]
            .iter()
            .for_each(|kv| lru.insert(kv.0, kv.1));
        let node = Rc::clone(lru.map.get(&3).unwrap());
        lru.detach(node);
        assert_eq!(lru.into_iter().collect::<Vec<_>>(), vec![(1, 2), (5, 6)]);
    }

    #[test]
    fn detach_tail() {
        let mut lru = SyncNaiveLru::new();
        vec![(1, 2), (3, 4), (5, 6)]
            .iter()
            .for_each(|kv| lru.insert(kv.0, kv.1));
        let node = Rc::clone(lru.map.get(&1).unwrap());
        lru.detach(node);
        assert_eq!(lru.into_iter().collect::<Vec<_>>(), vec![(3, 4), (5, 6)]);
    }
}
