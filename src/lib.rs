use slab::Slab;

#[derive(Debug)]
pub struct SlabLinkedList<T> {
    slab: Slab<Item<T>>,
    head: Option<usize>,
    tail: Option<usize>,
}

impl<T> Default for SlabLinkedList<T> {
    #[inline]
    fn default() -> Self {
        Self {
            slab: Default::default(),
            head: None,
            tail: None,
        }
    }
}

impl<T> SlabLinkedList<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.slab.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slab.is_empty()
    }

    #[inline]
    pub fn get(&self, key: usize) -> Option<&T> {
        self.slab.get(key).map(|item| &item.value)
    }

    #[inline]
    pub fn front(&self) -> Option<&T> {
        match self.head {
            None => None,
            Some(key) => {
                let item = unsafe { self.slab.get_unchecked(key) };
                Some(&item.value)
            }
        }
    }

    #[inline]
    pub fn back(&self) -> Option<&T> {
        match self.tail {
            None => None,
            Some(key) => {
                let item = unsafe { self.slab.get_unchecked(key) };
                Some(&item.value)
            }
        }
    }

    #[inline]
    #[track_caller]
    pub fn insert_before(&mut self, value: T, target_key: usize) -> usize {
        let key = self.slab.insert(Item::from(value));
        let (item, target_item) = self.slab.get2_mut(key, target_key).expect("invalid key");
        item.key.replace(key);
        item.next.replace(target_key);

        match target_item.prev.replace(key) {
            None => {
                // target is head
                assert_eq!(self.head.replace(key), Some(target_key));
            }
            Some(prev) => {
                let prev_item = self.slab.get_mut(prev).unwrap();
                assert_eq!(prev_item.next.replace(key), Some(target_key));
            }
        }

        key
    }

    #[inline]
    #[track_caller]
    pub fn insert_after(&mut self, value: T, target_key: usize) -> usize {
        let key = self.slab.insert(Item::from(value));
        let (item, target_item) = self.slab.get2_mut(key, target_key).expect("invalid key");
        item.key.replace(key);
        item.prev.replace(target_key);

        match target_item.next.replace(key) {
            None => {
                // target is tail
                assert_eq!(self.tail.replace(key), Some(target_key));
            }
            Some(next) => {
                let next_item = self.slab.get_mut(next).unwrap();
                assert_eq!(next_item.prev.replace(key), Some(target_key));
            }
        }

        key
    }

    #[inline]
    fn insert_first_item(&mut self, value: T) -> usize {
        assert!(self.slab.is_empty());
        let key = self.slab.insert(Item::from(value));
        assert_eq!(self.head.replace(key), None);
        assert_eq!(self.tail.replace(key), None);
        let item = self.slab.get_mut(key).unwrap();
        item.key.replace(key);
        key
    }

    #[inline]
    pub fn push_front(&mut self, value: T) -> usize {
        match self.head {
            None => self.insert_first_item(value),
            Some(target_key) => self.insert_before(value, target_key),
        }
    }

    #[inline]
    pub fn push_back(&mut self, value: T) -> usize {
        match self.tail {
            None => self.insert_first_item(value),
            Some(target_key) => self.insert_after(value, target_key),
        }
    }

    #[inline]
    #[track_caller]
    pub fn pop_front(&mut self) -> Option<T> {
        let key = self.head?;
        let value = self.remove(key);
        Some(value)
    }

    #[inline]
    #[track_caller]
    pub fn pop_back(&mut self) -> Option<T> {
        let key = self.tail?;
        let value = self.remove(key);
        Some(value)
    }

    #[inline]
    pub fn try_remove(&mut self, key: usize) -> Option<T> {
        let Some(item) = self.slab.try_remove(key) else {
            return None;
        };

        let Item {
            value,
            key: stored_key,
            prev,
            next,
        } = item;

        assert_eq!(stored_key, Some(key));

        match (prev, next) {
            (Some(prev), Some(next)) => {
                let (prev_item, next_item) = self.slab.get2_mut(prev, next).unwrap();
                assert_eq!(prev_item.next.replace(next), Some(key));
                assert_eq!(next_item.prev.replace(prev), Some(key));
            }
            (Some(prev), None) => {
                // is tail
                let prev_item = self.slab.get_mut(prev).unwrap();
                assert_eq!(prev_item.next.take(), Some(key));
                assert_eq!(self.tail.replace(prev), Some(key));
            }
            (None, Some(next)) => {
                // is head
                let next_item = self.slab.get_mut(next).unwrap();
                assert_eq!(next_item.prev.take(), Some(key));
                assert_eq!(self.head.replace(next), Some(key));
            }
            (None, None) => {
                // is only item
                assert_eq!(self.head.take(), Some(key));
                assert_eq!(self.tail.take(), Some(key));
            }
        }

        Some(value)
    }

    #[inline]
    #[track_caller]
    pub fn remove(&mut self, key: usize) -> T {
        self.try_remove(key).expect("invalid key")
    }
}

#[derive(Debug)]
struct Item<T> {
    value: T,
    key: Option<usize>,
    prev: Option<usize>,
    next: Option<usize>,
}

impl<T> From<T> for Item<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self {
            value,
            key: None,
            prev: None,
            next: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_front() {
        let mut list = SlabLinkedList::new();
        let i1 = list.push_front("a");
        assert_eq!(list.head, Some(i1));
        assert_eq!(list.tail, Some(i1));
        assert_eq!(list.front(), Some(&"a"));
        assert_eq!(list.slab.len(), 1);
        let i2 = list.push_front("b");
        assert_eq!(list.head, Some(i2));
        assert_eq!(list.tail, Some(i1));
        assert_eq!(list.front(), Some(&"b"));
        assert_eq!(list.back(), Some(&"a"));
        assert_eq!(list.slab.len(), 2);
    }

    #[test]
    fn push_back() {
        let mut list = SlabLinkedList::new();
        let i1 = list.push_back("a");
        assert_eq!(list.head, Some(i1));
        assert_eq!(list.tail, Some(i1));
        assert_eq!(list.slab.len(), 1);
        let i2 = list.push_back("b");
        assert_eq!(list.head, Some(i1));
        assert_eq!(list.tail, Some(i2));
        assert_eq!(list.slab.len(), 2);
    }

    #[test]
    fn remove() {
        let mut list = SlabLinkedList::new();
        let i1 = list.push_front("a");
        let i2 = list.push_front("b");
        let i3 = list.push_back("c");
        assert_eq!(list.head, Some(i2));
        assert_eq!(list.tail, Some(i3));

        assert_eq!(list.remove(i1), "a");
        assert_eq!(list.head, Some(i2));
        assert_eq!(list.tail, Some(i3));
        assert_eq!(list.slab.len(), 2);

        assert_eq!(list.remove(i2), "b");
        assert_eq!(list.head, Some(i3));
        assert_eq!(list.tail, Some(i3));
        assert_eq!(list.slab.len(), 1);

        assert_eq!(list.remove(i3), "c");
        assert_eq!(list.head, None);
        assert_eq!(list.tail, None);
        assert_eq!(list.slab.len(), 0);
    }

    #[test]
    fn pop_front() {
        let mut list = SlabLinkedList::new();
        list.push_front("a");
        list.push_front("b");
        list.push_back("c");
        // "b", "a", "c"
        assert_eq!(list.pop_front(), Some("b"));
        assert_eq!(list.pop_front(), Some("a"));
        assert_eq!(list.pop_front(), Some("c"));
    }

    #[test]
    fn pop_back() {
        let mut list = SlabLinkedList::new();
        list.push_front("a");
        list.push_front("b");
        list.push_back("c");
        // "b", "a", "c"
        assert_eq!(list.pop_back(), Some("c"));
        assert_eq!(list.pop_back(), Some("a"));
        assert_eq!(list.pop_back(), Some("b"));
    }
}
