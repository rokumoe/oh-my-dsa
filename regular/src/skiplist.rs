use std::cmp::Ordering;
use std::mem;
use std::ptr::NonNull;

const LEVELS: usize = 9;

#[derive(Debug)]
pub struct PseudoRand {
    s: u64,
}

impl PseudoRand {
    fn new(seed: u64) -> Self {
        Self { s: seed }
    }

    fn rand(&mut self, n: u64) -> u64 {
        let x = self.s.wrapping_mul(1103515245).wrapping_add(12345);
        self.s = x;
        x % n
    }
}

#[derive(Debug)]
struct Node {
    key: u64,
    forward: [Option<NonNull<Node>>; LEVELS],
}

impl Node {
    fn new(key: u64, f: Option<NonNull<Node>>) -> Self {
        Self {
            key,
            forward: [f; LEVELS],
        }
    }
}

#[derive(Debug)]
pub struct Skiplist {
    head: Node,
    level: usize,
    sentinel: Option<NonNull<Node>>,
    rnd: PseudoRand,
}

impl Drop for Skiplist {
    fn drop(&mut self) {
        let mut p = self.head.forward[0];
        while p != self.sentinel {
            let n = unsafe { Box::from_raw(p.unwrap().as_ptr()) };
            p = n.forward[0];
        }
        mem::drop(unsafe { Box::from_raw(self.sentinel.unwrap().as_ptr()) });
    }
}

impl Skiplist {
    pub fn new() -> Self {
        let sentinel = NonNull::new(Box::into_raw(Box::new(Node::new(0, None))));
        Self {
            head: Node::new(0, sentinel),
            level: 0,
            sentinel,
            rnd: PseudoRand::new(0),
        }
    }

    pub fn search(&self, key: u64) -> bool {
        let mut p = &self.head;
        let mut k = self.level;
        loop {
            loop {
                if p.forward[k] == self.sentinel {
                    break;
                }
                let q = unsafe { &*p.forward[k].unwrap().as_ptr() };
                match q.key.cmp(&key) {
                    Ordering::Less => p = q,
                    Ordering::Equal => return true,
                    Ordering::Greater => break,
                }
            }
            if k == 0 {
                break;
            }
            k -= 1;
        }
        false
    }

    fn locate(
        &self,
        key: u64,
        update: &mut [Option<NonNull<Node>>; LEVELS],
    ) -> Option<NonNull<Node>> {
        let mut p = &self.head;
        let mut k = self.level;
        let mut found = false;
        let mut q = None;
        loop {
            while p.forward[k] != self.sentinel {
                q = p.forward[k];
                let x = unsafe { &*q.unwrap().as_ptr() };
                match x.key.cmp(&key) {
                    Ordering::Less => p = x,
                    Ordering::Equal => {
                        found = true;
                        break;
                    }
                    Ordering::Greater => break,
                }
            }
            update[k] = NonNull::new(p as *const _ as *mut _);
            if k == 0 {
                break;
            }
            k -= 1;
        }
        if found {
            q
        } else {
            None
        }
    }

    fn pick_level(&mut self) -> usize {
        assert!(LEVELS < 32);
        (self.rnd.rand(1 << (LEVELS + LEVELS)).trailing_zeros() / 2) as usize
    }

    pub fn insert(&mut self, key: u64) -> bool {
        let mut update = [None; LEVELS];
        let found = self.locate(key, &mut update);
        if found.is_some() {
            return true;
        }
        let mut k = self.pick_level();
        if k > self.level {
            k = self.level + 1;
            self.level = k;
            update[k] = NonNull::new(&mut self.head as *mut _);
        }
        let mut x = Box::new(Node::new(key, None));
        let q = NonNull::new(x.as_mut() as *mut _);
        loop {
            let p = unsafe { &mut *update[k].unwrap().as_ptr() };
            x.forward[k] = p.forward[k];
            p.forward[k] = q;
            if k == 0 {
                break;
            }
            k -= 1;
        }
        mem::forget(x);
        false
    }

    pub fn remove(&mut self, key: u64) -> bool {
        let mut update = [None; LEVELS];
        let found = self.locate(key, &mut update);
        if found.is_none() {
            return false;
        }
        let q = unsafe { Box::from_raw(found.unwrap().as_ptr()) };
        for k in 0..=self.level {
            let p = unsafe { &mut *update[k].unwrap().as_ptr() };
            if p.forward[k] != found {
                break;
            }
            p.forward[k] = q.forward[k];
        }
        let mut k = self.level;
        while self.head.forward[k] == self.sentinel && k > 0 {
            k -= 1;
        }
        self.level = k;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut sl = Skiplist::new();
        sl.insert(1);
        sl.insert(4);
        sl.insert(2);
        sl.insert(3);
        assert!(!sl.search(0));
        assert!(sl.search(1));
        assert!(sl.search(4));
        assert!(!sl.search(5));
    }

    #[test]
    fn test_remove() {
        let mut sl = Skiplist::new();
        sl.insert(1);
        sl.insert(4);
        sl.insert(2);
        sl.insert(3);
        assert!(sl.remove(1));
        assert!(sl.remove(4));
        assert!(sl.remove(2));
        assert!(sl.remove(3));
        assert!(!sl.remove(1));
    }
}
