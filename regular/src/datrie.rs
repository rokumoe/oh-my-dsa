// https://github.com/komiya-atsushi/darts-java/blob/master/src/main/java/darts/DoubleArrayTrie.java

pub struct DATrie {
    base: Vec<isize>,
    check: Vec<isize>,
}

impl DATrie {
    pub fn contains(&self, key: &str) -> bool {
        let mut s = self.base[0];
        for &c in key.as_bytes() {
            let t = s + c as isize + 1;
            if t < self.check.len() as isize && self.check[t as usize] == s {
                s = self.base[t as usize];
            } else {
                return false;
            }
        }
        let end = self.base[s as usize];
        self.check[s as usize] == s && end < 0
    }

    pub fn dump(&self) {
        println!("i\tbase\tcheck");
        for i in 0..self.base.len() {
            if self.base[i] != 0 || self.check[i] != 0 {
                println!("{}\t{}\t{}", i, self.base[i], self.check[i]);
            }
        }
    }
}

struct Node {
    code: usize,
    depth: usize,
    left: usize,
    right: usize,
}

impl Node {
    fn new(code: usize, depth: usize, left: usize, right: usize) -> Self {
        Node {
            code,
            depth,
            left,
            right,
        }
    }
}

pub struct Builder {
    keys: Vec<String>,
    base: Vec<isize>,
    check: Vec<isize>,
    used: Vec<bool>,
    next_check: usize,
    alloc_size: usize,
    max_size: usize,
    progress: usize,
}

impl Builder {
    fn new(keys: Vec<String>) -> Self {
        Builder {
            keys,
            base: vec![1],
            check: vec![0],
            used: vec![false],
            next_check: 0,
            alloc_size: 1,
            max_size: 1,
            progress: 0,
        }
    }

    fn fetch(&self, parent: &Node, siblings: &mut Vec<Node>) {
        let mut prev = 0usize;
        for i in parent.left..parent.right {
            let key = self.keys[i].as_bytes();
            if key.len() < parent.depth {
                continue;
            }
            let mut cur = 0usize;
            if key.len() > parent.depth {
                cur = key[parent.depth] as usize + 1;
            }
            assert!(cur >= prev);
            if cur > prev || siblings.len() == 0 {
                let n = Node::new(cur, parent.depth + 1, i, 0);
                siblings.last_mut().map(|last| last.right = i);
                siblings.push(n);
            }
            prev = cur;
        }
        siblings.last_mut().map(|last| last.right = parent.right);
    }

    fn resize(&mut self, new_size: usize) {
        if new_size < self.alloc_size {
            return;
        }
        self.base.resize(new_size, 0);
        self.check.resize(new_size, 0);
        self.used.resize(new_size, false);
        self.alloc_size = new_size;
    }

    fn insert(&mut self, siblings: &[Node]) -> usize {
        let mut pos = self.next_check.max(siblings[0].code + 1) - 1;
        let mut nonzero_num = 0;
        let mut more = false;
        let mut begin;
        loop {
            pos += 1;
            if self.alloc_size <= pos {
                self.resize(pos + 1);
            }
            if self.check[pos] != 0 {
                nonzero_num += 1;
                continue;
            } else if !more {
                self.next_check = pos;
                more = true;
            }
            begin = pos - siblings[0].code;
            if self.alloc_size <= begin + siblings.last().unwrap().code {
                let l = 1.05f64.max(self.keys.len() as f64 / (self.progress + 1) as f64);
                self.resize((self.alloc_size as f64 * l) as usize);
            }
            if self.used[begin] {
                continue;
            }
            let all_empty = siblings[1..]
                .iter()
                .all(|n| self.check[begin + n.code] == 0);
            if all_empty {
                break;
            }
        }

        // -- Simple heuristics --
        // if the percentage of non-empty contents in check between the
        // index
        // 'next_check_pos' and 'check' is greater than some constant value
        // (e.g. 0.9),
        // new 'next_check_pos' index is written by 'check'.
        if 1.0 * nonzero_num as f64 / (pos - self.next_check + 1) as f64 >= 0.95 {
            self.next_check = pos;
        }
        self.used[begin] = true;
        self.max_size = self.max_size.max(begin + siblings.last().unwrap().code + 1);
        for n in siblings {
            self.check[begin + n.code] = begin as isize;
        }
        let mut new_siblings = Vec::new();
        for n in siblings {
            new_siblings.clear();
            self.fetch(n, &mut new_siblings);
            if new_siblings.is_empty() {
                self.base[begin + n.code] = -1 - n.left as isize;
                self.progress += 1;
            } else {
                let h = self.insert(&new_siblings);
                self.base[begin + n.code] = h as isize;
            }
        }
        begin
    }

    fn into_datrie(self) -> DATrie {
        let mut base = self.base;
        base.resize(self.max_size, 0);
        base.shrink_to_fit();
        let mut check = self.check;
        check.resize(self.max_size, 0);
        check.shrink_to_fit();
        DATrie { base, check }
    }
}

pub fn build(keys: Vec<String>) -> DATrie {
    let mut builder = Builder::new(keys);
    let mut siblings = Vec::new();
    builder.fetch(&Node::new(0, 0, 0, builder.keys.len()), &mut siblings);
    builder.insert(&siblings);
    builder.into_datrie()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let mut dict = vec!["AC", "ACE", "ACFF", "AD", "CD", "CF", "ZQ"];
        dict.sort();
        let dict = dict.iter().map(|s| s.to_string()).collect();
        let dat = build(dict);
        dat.dump();
        assert!(dat.contains("ACFF"));
        assert!(!dat.contains("ZZZ"));
    }
}
