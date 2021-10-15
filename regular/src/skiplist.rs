use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Bound, RangeBounds};
use std::ptr::NonNull;

use crate::rand;

type NodePtr<K, V, const L: usize> = Option<NonNull<Node<K, V, L>>>;

struct Node<K, V, const L: usize> {
    next: [NodePtr<K, V, L>; L],
    key: K,
    value: V,
}

impl<K, V, const L: usize> Node<K, V, L> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            next: [None; L],
        }
    }
}

pub struct Skiplist<K, V, const L: usize> {
    head: [NodePtr<K, V, L>; L],
    level: usize,
}

impl<K, V, const L: usize> Drop for Skiplist<K, V, L> {
    fn drop(&mut self) {
        self.drain();
    }
}

impl<K, V, const L: usize> Skiplist<K, V, L> {
    pub fn new() -> Self {
        Self {
            head: [None; L],
            level: 0,
        }
    }

    fn rand_level() -> usize {
        assert!(L < 64);
        (rand::random_u64() % (1u64 << L)).trailing_zeros() as usize
    }

    fn link_node(&mut self, preds: &mut [NodePtr<K, V, L>; L], node: NodePtr<K, V, L>) {
        let mut new_level = Self::rand_level();
        if new_level > self.level {
            new_level = self.level + 1;
            self.level = new_level;
            preds[new_level] = None;
        }
        let curr = unsafe { node.unwrap().as_mut() };
        for k in 0..=new_level {
            if let Some(mut p) = preds[k] {
                let pred = unsafe { p.as_mut() };
                curr.next[k] = pred.next[k];
                pred.next[k] = node;
            } else {
                curr.next[k] = self.head[k];
                self.head[k] = node;
            }
        }
    }

    fn shrink_level(&mut self) {
        let mut k = self.level;
        while k > 0 && self.head[k].is_none() {
            k -= 1;
        }
        self.level = k;
    }

    fn remove_node(&mut self, preds: &[NodePtr<K, V, L>; L], node: NodePtr<K, V, L>) {
        let curr = unsafe { node.unwrap().as_ref() };
        for k in 0..=self.level {
            if let Some(mut p) = preds[k] {
                let pred = unsafe { p.as_mut() };
                if pred.next[k] == node {
                    pred.next[k] = curr.next[k];
                } else {
                    break;
                }
            } else if self.head[k] == node {
                self.head[k] = curr.next[k];
            } else {
                break;
            }
        }
        self.shrink_level();
    }

    pub fn drain(&mut self) -> Drain<K, V, L> {
        let p = self.head[0].take();
        self.level = 0;
        Drain { curr: p }
    }
}

impl<K, V, const L: usize> Skiplist<K, V, L>
where
    K: Ord,
{
    fn search_node<Q: ?Sized>(&self, key: &Q) -> Result<NodePtr<K, V, L>, NodePtr<K, V, L>>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        let mut prev = None;
        let mut link = &self.head;
        for k in (0..=self.level).rev() {
            while let Some(n) = link[k] {
                let curr = unsafe { n.as_ref() };
                match curr.key.borrow().cmp(key) {
                    Ordering::Less => {
                        prev = Some(n);
                        link = &curr.next;
                    }
                    Ordering::Equal => return Ok(Some(n)),
                    Ordering::Greater => break,
                }
            }
        }
        Err(prev)
    }

    fn search_preds<Q: ?Sized>(
        &self,
        key: &Q,
        preds: &mut [NodePtr<K, V, L>; L],
        exclude: bool,
    ) -> NodePtr<K, V, L>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        let mut prev = None;
        let mut found = None;
        let mut next = &self.head;
        for k in (0..=self.level).rev() {
            while let Some(n) = next[k] {
                let curr = unsafe { n.as_ref() };
                match curr.key.borrow().cmp(&key) {
                    Ordering::Greater => break,
                    Ordering::Equal if !exclude => {
                        found = Some(n);
                        break;
                    }
                    _ => {
                        prev = Some(n);
                        next = &curr.next;
                    }
                }
            }
            preds[k] = prev;
        }
        found
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        if let Ok(Some(n)) = self.search_node(key) {
            Some(unsafe { &n.as_ref().value })
        } else {
            None
        }
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        if let Ok(Some(mut p)) = self.search_node(key) {
            Some(unsafe { &mut p.as_mut().value })
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut preds = [None; L];
        let found = self.search_preds(&key, &mut preds, false);
        if let Some(mut n) = found {
            Some(mem::replace(&mut unsafe { n.as_mut() }.value, value))
        } else {
            self.link_node(
                &mut preds,
                NonNull::new(Box::into_raw(Box::new(Node::new(key, value)))),
            );
            None
        }
    }

    pub fn insert_cow(&mut self, key: Cow<K>, value: V) -> Option<V>
    where
        K: ToOwned<Owned = K>,
    {
        let mut preds = [None; L];
        let found = self.search_preds(key.borrow(), &mut preds, false);
        if let Some(mut n) = found {
            Some(mem::replace(&mut unsafe { n.as_mut() }.value, value))
        } else {
            self.link_node(
                &mut preds,
                NonNull::new(Box::into_raw(Box::new(Node::new(key.into_owned(), value)))),
            );
            None
        }
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        let mut preds = [None; L];
        self.search_preds(&key, &mut preds, false).map(|found| {
            self.remove_node(&preds, Some(found));
            unsafe { Box::from_raw(found.as_ptr()) }.value
        })
    }

    fn range_bound_ptr<T: ?Sized, R>(&self, range: R) -> RangePtr<K, V, L>
    where
        T: Ord,
        K: Borrow<T>,
        R: RangeBounds<T>,
    {
        if !check_range(&range) {
            return RangePtr {
                curr: None,
                end: None,
            };
        }
        let start_bound = range.start_bound();
        let start = match start_bound {
            Bound::Included(key) | Bound::Excluded(key) => {
                let (found, mut p) = self
                    .search_node(key)
                    .map_or_else(|e| (false, e), |n| (true, n));
                if matches!(start_bound, Bound::Excluded(_)) || !found {
                    if let Some(n) = p {
                        p = unsafe { n.as_ref() }.next[0];
                    }
                }
                p
            }
            Bound::Unbounded => self.head[0],
        };
        let end_bound = range.end_bound();
        let end = match end_bound {
            Bound::Included(key) | Bound::Excluded(key) => {
                let (found, mut p) = self
                    .search_node(key)
                    .map_or_else(|e| (false, e), |n| (true, n));
                if matches!(end_bound, Bound::Included(_)) || !found {
                    if let Some(n) = p {
                        p = unsafe { n.as_ref() }.next[0];
                    }
                }
                p
            }
            Bound::Unbounded => None,
        };
        RangePtr { curr: start, end }
    }

    pub fn range<T: ?Sized, R>(&self, range: R) -> Range<'_, K, V, L>
    where
        T: Ord,
        K: Borrow<T>,
        R: RangeBounds<T>,
    {
        Range {
            inner: self.range_bound_ptr(range),
            _marker: PhantomData,
        }
    }

    pub fn range_mut<T: ?Sized, R>(&self, range: R) -> RangeMut<'_, K, V, L>
    where
        T: Ord,
        K: Borrow<T>,
        R: RangeBounds<T>,
    {
        RangeMut {
            inner: self.range_bound_ptr(range),
            _marker: PhantomData,
        }
    }

    pub fn drain_range<T: ?Sized, R>(&mut self, range: R) -> Drain<K, V, L>
    where
        T: Ord,
        K: Borrow<T>,
        R: RangeBounds<T>,
    {
        if !check_range(&range) {
            return Drain { curr: None };
        }

        let start_bound = range.start_bound();
        let mut start_preds = [None; L];
        let start = match start_bound {
            Bound::Included(key) | Bound::Excluded(key) => {
                let p = self.search_preds(
                    key,
                    &mut start_preds,
                    matches!(start_bound, Bound::Excluded(_)),
                );
                if p.is_none() {
                    start_preds[0].and_then(|n| unsafe { n.as_ref().next[0] })
                } else {
                    p
                }
            }
            Bound::Unbounded => None,
        };

        let end_bound = range.end_bound();
        let mut end_preds = [None; L];
        let end = match end_bound {
            Bound::Included(key) | Bound::Excluded(key) => {
                let p =
                    self.search_preds(key, &mut end_preds, matches!(end_bound, Bound::Included(_)));
                if p.is_none() {
                    end_preds[0].and_then(|n| unsafe { n.as_ref().next[0] })
                } else {
                    p
                }
            }
            Bound::Unbounded => None,
        };

        if start == None && end == None {
            return self.drain();
        }

        for k in 0..=self.level {
            let next = end_preds[k].and_then(|mut n| unsafe { n.as_mut().next[k].take() });
            if let Some(mut n) = start_preds[k] {
                unsafe { n.as_mut().next[k] = next };
            } else {
                self.head[k] = next;
            }
        }
        self.shrink_level();

        Drain { curr: start }
    }
}

struct RangePtr<K, V, const L: usize> {
    curr: NodePtr<K, V, L>,
    end: NodePtr<K, V, L>,
}

impl<K, V, const L: usize> RangePtr<K, V, L> {
    fn next_ptr(&mut self) -> NodePtr<K, V, L> {
        if self.curr != self.end {
            if let Some(n) = self.curr {
                let curr = unsafe { n.as_ref() };
                self.curr = curr.next[0];
                return Some(n);
            }
        }
        None
    }
}

pub struct Range<'a, K, V, const L: usize> {
    inner: RangePtr<K, V, L>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, K: 'a, V: 'a, const L: usize> Iterator for Range<'a, K, V, L> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_ptr().map(|n| {
            let curr = unsafe { n.as_ref() };
            (&curr.key, &curr.value)
        })
    }
}

pub struct RangeMut<'a, K, V, const L: usize> {
    inner: RangePtr<K, V, L>,
    _marker: PhantomData<&'a mut ()>,
}

impl<'a, K: 'a, V: 'a, const L: usize> Iterator for RangeMut<'a, K, V, L> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_ptr().map(|mut n| {
            let curr = unsafe { n.as_mut() };
            (&curr.key, &mut curr.value)
        })
    }
}

pub struct Drain<K, V, const L: usize> {
    curr: NodePtr<K, V, L>,
}

impl<K, V, const L: usize> Iterator for Drain<K, V, L> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(n) = self.curr {
            let curr_node = unsafe { Box::from_raw(n.as_ptr()) };
            self.curr = curr_node.next[0];
            Some((curr_node.key, curr_node.value))
        } else {
            None
        }
    }
}

impl<K, V, const L: usize> Drop for Drain<K, V, L> {
    fn drop(&mut self) {
        let mut curr = self.curr;
        while let Some(n) = curr {
            let curr_node = unsafe { Box::from_raw(n.as_ptr()) };
            curr = curr_node.next[0];
        }
    }
}

fn check_range<T: ?Sized, R>(range: &R) -> bool
where
    T: Ord,
    R: RangeBounds<T>,
{
    let start = range.start_bound();
    let end = range.end_bound();
    if start == end {
        return true;
    }
    let start_key = match start {
        Bound::Included(key) | Bound::Excluded(key) => key,
        Bound::Unbounded => return true,
    };
    let end_key = match end {
        Bound::Included(key) | Bound::Excluded(key) => key,
        Bound::Unbounded => return true,
    };
    match start_key.cmp(&end_key) {
        Ordering::Equal => matches!(start, Bound::Excluded(_)) || matches!(end, Bound::Included(_)),
        r => r.is_le(),
    }
}
