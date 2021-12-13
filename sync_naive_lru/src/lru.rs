use std::{
    borrow::Borrow,
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
    pub(crate) key: Rc<K>,
    pub(crate) value: V,
}

impl<K, V> Node<K, V> {
    pub fn new(key: Rc<K>, value: V) -> Self {
        Self {
            next: None,
            prev: None,
            key,
            value,
        }
    }
}

/// LRU cache implemented by hash map and doubly-linked list.
/// more recently accessed element lies head of the list and least recently accessed one lies the
/// opposite.
pub struct SyncNaiveLru<K, V> {
    map: HashMap<Rc<K>, NodeRef<K, V>>,
    head: Option<NodeRef<K, V>>,
    pub(crate) tail: Option<NodeRef<K, V>>,
    capacity: usize,
}

impl<K, V> SyncNaiveLru<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            head: None,
            tail: None,
            capacity,
        }
    }

    /// Insert a new key-value pair.
    /// If the number of existing elements is `capacity`, remove least-recently accessed one.
    pub fn insert(&mut self, key: K, value: V) {
        let key = Rc::new(key);
        let node = Rc::new(RefCell::new(Node::new(Rc::clone(&key), value)));
        self.map.insert(key, Rc::clone(&node));
        self.attach(node);

        if self.map.len() == self.capacity + 1 {
            let tail = self.tail.clone().expect("There must be at least 1 element");
            self.map.remove(&tail.as_ref().borrow().key);
            self.detach(tail);
        }
    }

    /// Get clone of a value corresponding to `key`.
    /// This requires mutable reference to `self` because this modifies the order of inner
    /// elements; moves accessed element to head of the list.
    pub fn get<Q>(&mut self, key: &Q) -> Option<V>
    where
        Rc<K>: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        if let Some(node) = self.map.get(key).cloned() {
            self.detach(Rc::clone(&node));
            self.attach(Rc::clone(&node));
            return Some(node.as_ref().borrow().value.clone());
        }
        None
    }

    /// Attach `node` to the head of linked list.
    fn attach(&mut self, node: NodeRef<K, V>) {
        if self.head.is_some() {
            (*node.borrow_mut()).prev = Some(Rc::downgrade(self.head.as_ref().unwrap()));
            (*node.borrow_mut()).next = None;
            (*self.head.as_ref().unwrap().borrow_mut()).next = Some(Rc::clone(&node));
        } else {
            self.tail = Some(Rc::clone(&node));
        }
        self.head = Some(node);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_lru_with_capacity_3() -> SyncNaiveLru<i32, i32> {
        let mut lru = SyncNaiveLru::new(3);
        vec![(1, 2), (3, 4), (5, 6)]
            .iter()
            .for_each(|kv| lru.insert(kv.0, kv.1));
        lru
    }

    #[test]
    fn just_insert() {
        let lru = setup_lru_with_capacity_3();
        let tail = lru.tail.as_ref().unwrap();
        assert_eq!(
            (
                tail.as_ref().borrow().key.clone(),
                tail.as_ref().borrow().value
            ),
            (Rc::new(1), 2)
        );
        assert_eq!(
            lru.into_iter().collect::<Vec<_>>(),
            vec![(1, 2), (3, 4), (5, 6)]
        );
    }

    #[test]
    fn detach_head() {
        let mut lru = setup_lru_with_capacity_3();
        let node = Rc::clone(lru.map.get(&5).unwrap());
        lru.detach(node);
        assert_eq!(lru.into_iter().collect::<Vec<_>>(), vec![(1, 2), (3, 4)]);
    }

    #[test]
    fn detach_middle() {
        let mut lru = setup_lru_with_capacity_3();
        let node = Rc::clone(lru.map.get(&3).unwrap());
        lru.detach(node);
        assert_eq!(lru.into_iter().collect::<Vec<_>>(), vec![(1, 2), (5, 6)]);
    }

    #[test]
    fn detach_tail() {
        let mut lru = setup_lru_with_capacity_3();
        let node = Rc::clone(lru.map.get(&1).unwrap());
        lru.detach(node);
        assert_eq!(lru.into_iter().collect::<Vec<_>>(), vec![(3, 4), (5, 6)]);
    }

    #[test]
    fn exceeding_insert() {
        let mut lru = SyncNaiveLru::new(3);
        let expected = vec![(1, 2), (3, 4), (5, 6), (7, 8)];
        expected.iter().for_each(|kv| lru.insert(kv.0, kv.1));

        assert_eq!(lru.get(&1), None);
        assert_eq!(
            lru.into_iter().collect::<Vec<_>>(),
            vec![(3, 4), (5, 6), (7, 8)]
        );
    }

    #[test]
    fn get_reorders_entry() {
        let mut lru = setup_lru_with_capacity_3();
        assert_eq!(lru.get(&3), Some(4));
        assert_eq!(
            lru.into_iter().collect::<Vec<_>>(),
            vec![(1, 2), (5, 6), (3, 4)]
        );
    }
}
