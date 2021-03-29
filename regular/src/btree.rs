// https://www.geeksforgeeks.org/introduction-of-b-tree-2/
// https://www.geeksforgeeks.org/b-tree-set-1-insert-2/
// https://www.geeksforgeeks.org/delete-operation-in-b-tree/

use std::cmp::Ordering;
use std::mem;

// B-Tree 属性：
// - leaf 在同一层
// - 最小度 t
// - 根节点至少有一个 key
// - 非根节点至少 t-1 个 key
// - 所有节点至多 2t-1 个 key
// - 节点 children 数等于 key 数量+1
// - 节点 keys 有序

const T: usize = 3;

#[derive(Debug)]
struct Node {
    keys: Vec<i32>,
    children: Vec<Box<Node>>,
}

impl Node {
    fn new() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn is_full(&self) -> bool {
        self.keys.len() == 2 * T - 1
    }

    fn search_in_node(&self, k: &i32) -> Result<usize, usize> {
        let mut p: usize = 0;
        for (i, key) in self.keys.iter().enumerate() {
            match key.cmp(&k) {
                Ordering::Equal => return Ok(i),
                Ordering::Greater => break,
                _ => p = i + 1,
            }
        }
        Err(p)
    }

    fn split_child(&mut self, at: usize) {
        let y = &mut self.children[at];
        let mut z = Box::new(Node::new());
        z.keys = y.keys.drain(T..).collect();
        if !y.is_leaf() {
            z.children = y.children.drain(T..).collect();
        }
        let key = y.keys.pop().unwrap();
        self.keys.insert(at, key);
        self.children.insert(at + 1, z);
    }

    fn insert_non_full(&mut self, k: i32) -> bool {
        let mut p = self.search_in_node(&k).unwrap_or_else(|e| e);
        if self.is_leaf() {
            self.keys.insert(p, k);
            false
        } else {
            if self.children[p].is_full() {
                self.split_child(p);
                if matches!(self.keys[p].cmp(&k), Ordering::Less) {
                    p += 1;
                }
            }
            self.children[p].insert_non_full(k)
        }
    }

    fn remove_leaf(&mut self, at: usize) -> bool {
        self.keys.remove(at);
        true
    }

    fn predecessor(&self, at: usize) -> i32 {
        let mut cur = &self.children[at];
        while !cur.is_leaf() {
            cur = &cur.children[cur.keys.len()];
        }
        *cur.keys.last().unwrap()
    }

    fn successor(&self, at: usize) -> i32 {
        let mut cur = &self.children[at];
        while !cur.is_leaf() {
            cur = &cur.children[0];
        }
        *cur.keys.first().unwrap()
    }

    fn merge(&mut self, at: usize) {
        let sibling = self.children.remove(at + 1);
        let key = self.keys.remove(at);
        let child = &mut self.children[at];
        child.keys.push(key);
        child.keys.extend(sibling.keys);
        if !child.is_leaf() {
            child.children.extend(sibling.children);
        }
    }

    fn remove_non_leaf(&mut self, at: usize) -> bool {
        let key = self.keys[at];
        if self.children[at].keys.len() >= T {
            let pred = self.predecessor(at);
            self.keys[at] = pred;
            self.children[at].remove(&pred)
        } else if self.children[at + 1].keys.len() >= T {
            let succ = self.successor(at);
            self.keys[at] = succ;
            self.children[at + 1].remove(&succ)
        } else {
            self.merge(at);
            self.children[at].remove(&key)
        }
    }

    fn borrow_prev(&mut self, at: usize) {
        let sibling = &mut self.children[at - 1];
        let last_key = sibling.keys.pop().unwrap();
        let last_child = sibling.children.pop();
        let child = &mut self.children[at];
        child
            .keys
            .insert(0, mem::replace(&mut self.keys[at - 1], last_key));
        if let Some(c) = last_child {
            child.children.insert(0, c);
        }
    }

    fn borrow_next(&mut self, at: usize) {
        let sibling = &mut self.children[at + 1];
        let next_key = sibling.keys.remove(0);
        let next_child = if !sibling.children.is_empty() {
            Some(sibling.children.remove(0))
        } else {
            None
        };
        let child = &mut self.children[at];
        child.keys.push(mem::replace(&mut self.keys[at], next_key));
        if let Some(c) = next_child {
            child.children.push(c);
        }
    }

    fn fill(&mut self, at: usize) {
        if at > 0 && self.children[at - 1].keys.len() >= T {
            self.borrow_prev(at);
        } else if at < self.keys.len() && self.children[at + 1].keys.len() >= T {
            self.borrow_next(at);
        } else if at < self.keys.len() {
            self.merge(at);
        } else {
            self.merge(at - 1);
        }
    }

    fn remove(&mut self, k: &i32) -> bool {
        match self.search_in_node(k) {
            Ok(p) => {
                if self.is_leaf() {
                    self.remove_leaf(p)
                } else {
                    self.remove_non_leaf(p)
                }
            }
            Err(p) => {
                if !self.is_leaf() {
                    let is_last = self.keys.len() == p;
                    if self.children[p].keys.len() < T {
                        self.fill(p);
                    }
                    if is_last && p > self.keys.len() {
                        self.children[p - 1].remove(k)
                    } else {
                        self.children[p].remove(k)
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct BTree {
    root: Option<Box<Node>>,
}

impl BTree {
    pub fn new() -> Self {
        BTree { root: None }
    }

    pub fn search(&self, k: &i32) -> bool {
        let mut cur = if let Some(root) = &self.root {
            root
        } else {
            return false;
        };
        loop {
            let mut p = 0usize;
            while let Some(key) = cur.keys.get(p) {
                match key.cmp(k) {
                    Ordering::Less => p += 1,
                    Ordering::Equal => return true,
                    Ordering::Greater => break,
                }
            }
            if cur.is_leaf() {
                return false;
            }
            cur = &cur.children[p];
        }
    }

    pub fn insert(&mut self, k: i32) -> bool {
        if let Some(ref mut root) = self.root {
            if root.is_full() {
                let mut n = Box::new(Node::new());
                n.children.push(self.root.take().unwrap());
                n.split_child(0);
                let r = n.insert_non_full(k);
                self.root = Some(n);
                r
            } else {
                root.insert_non_full(k)
            }
        } else {
            let mut new_node = Box::new(Node::new());
            new_node.keys.push(k);
            self.root = Some(new_node);
            false
        }
    }

    pub fn remove(&mut self, k: &i32) -> bool {
        if let Some(ref mut root) = self.root {
            let r = root.remove(k);
            if root.keys.is_empty() {
                if root.is_leaf() {
                    self.root = None;
                } else {
                    self.root = Some(root.children.remove(0));
                }
            }
            r
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut t = BTree::new();
        t.insert(10);
        t.insert(20);
        t.insert(5);
        t.insert(6);
        t.insert(12);
        t.insert(30);
        t.insert(7);
        t.insert(17);
        t.insert(40);
        // println!("{:?}", t);
        t.insert(50);
        // println!("{:?}", t);
        println!("{}", t.search(&123));
        println!("{}", t.search(&17));
        println!("{}", t.search(&20));
        println!("{}", t.search(&50));
    }


    #[test]
    fn test_delete() {
        let mut t = BTree::new();
        t.insert(1);
        t.insert(2);
        t.insert(3);
        t.insert(4);
        t.insert(5);
        t.insert(6);
        println!("{:?}", t);
        println!("{}", t.remove(&1));
        println!("{}", t.remove(&2));
        println!("{}", t.remove(&3));
        println!("{}", t.remove(&4));
        println!("{}", t.remove(&5));
        println!("{}", t.remove(&6));
        println!("{:?}", t);
    }
}
