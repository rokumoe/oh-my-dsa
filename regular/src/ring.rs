use std::mem::{ManuallyDrop, MaybeUninit};

pub struct Iter<'a, T, const N: usize> {
    ring: &'a Ring<T, N>,
    i: usize,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.ring.tail {
            let v = &self.ring.items[self.i % N];
            self.i = self.i.wrapping_add(1);
            Some(&**v)
        } else {
            None
        }
    }
}

pub struct Ring<T, const N: usize> {
    tail: usize,
    head: usize,
    items: [ManuallyDrop<T>; N],
}

impl<T, const N: usize> Drop for Ring<T, N> {
    fn drop(&mut self) {
        for i in self.head..self.tail {
            unsafe {
                ManuallyDrop::drop(&mut self.items[i % N]);
            }
        }
    }
}

impl<T, const N: usize> Ring<T, N> {
    // 'const' modifier requires #![feature(const_maybe_uninit_assume_init)]
    pub fn new() -> Self {
        let items = unsafe { MaybeUninit::<[ManuallyDrop<T>; N]>::uninit().assume_init() };
        Self {
            tail: 0,
            head: 0,
            items,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tail.wrapping_sub(self.head)
    }

    pub fn is_full(&self) -> bool {
        self.len() == N
    }

    pub fn drop_front(&mut self) -> bool {
        if self.head < self.tail {
            unsafe {
                ManuallyDrop::drop(&mut self.items[self.head % N]);
            }
            self.head = self.head.wrapping_add(1);
            true
        } else {
            false
        }
    }

    pub fn push_back(&mut self, v: T) {
        if N == 0 {
            return;
        }
        if self.is_full() {
            self.drop_front();
        }
        self.items[self.tail % N] = ManuallyDrop::new(v);
        self.tail = self.tail.wrapping_add(1);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        (self.head < self.tail).then(|| {
            let v = unsafe { ManuallyDrop::take(&mut self.items[self.head % N]) };
            self.head = self.head.wrapping_add(1);
            v
        })
    }

    pub fn front(&self) -> Option<&T> {
        (self.head < self.tail).then(|| &*self.items[self.head % N])
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.head < self.tail {
            Some(&mut *self.items[self.head % N])
        } else {
            None
        }
    }

    pub fn get(&self, at: usize) -> Option<&T> {
        (at < self.len()).then(|| &*self.items[self.head.wrapping_add(at) % N])
    }

    pub fn get_mut(&mut self, at: usize) -> Option<&mut T> {
        if at < self.len() {
            Some(&mut *self.items[self.head.wrapping_add(at) % N])
        } else {
            None
        }
    }

    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter {
            ring: self,
            i: self.head,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Num(i32);

    impl Drop for Num {
        fn drop(&mut self) {
            println!("> drop {}", self.0);
        }
    }

    #[test]
    fn test_ring() {
        let mut ring = Ring::<Num, 3>::new();
        ring.push_back(Num(1));
        ring.push_back(Num(2));
        ring.push_back(Num(3));
        assert!(ring.is_full());
        println!("push_back 4");
        ring.push_back(Num(4));
        assert_eq!(ring.len(), 3);
        println!("pop_front");
        assert_eq!(ring.pop_front(), Some(Num(2)));
        println!("drop_front");
        assert!(ring.drop_front());
        assert_eq!(ring.len(), 1);
        assert_eq!(ring.get(0), Some(&Num(4)));
        println!("front {:?}", ring.front());
        assert_eq!(ring.get(1), None);
        println!("drop ring");
    }
}
