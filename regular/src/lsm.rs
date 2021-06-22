use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::mem;

use super::memfs::*;

const LSM_ROOT: &'static str = "/lsm";
const MAX_MEM_KV_NUM: usize = 4;
const MAX_BLOCK_KV_NUM: usize = 4;

struct Block {
    entries: Vec<(String, String)>,
}

impl Block {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn load(name: &str) -> Result<Self, String> {
        let mut f = Fs::open(name, false)?;
        let mut entries = Vec::new();
        loop {
            let key = match ioutil::read_string(&mut f) {
                Ok(k) => k,
                Err(e) if e == "EOF" => break,
                Err(e) => return Err(e),
            };
            let val = ioutil::read_string(&mut f)?;
            entries.push((key, val));
        }
        Ok(Block { entries })
    }

    fn load_first_key(name: &str) -> Result<String, String> {
        ioutil::read_string(&mut Fs::open(name, false)?)
    }

    fn write_into(self, name: &str) -> Result<(), String> {
        let mut d = Vec::new();
        for (k, v) in self.entries {
            ioutil::write_str(&mut d, &k);
            ioutil::write_str(&mut d, &v);
        }
        let mut f = Fs::create(name, true)?;
        f.write(&d);
        Ok(())
    }

    fn search(&self, key: &str) -> Option<&str> {
        self.entries
            .binary_search_by(|x| x.0.as_str().cmp(key))
            .ok()
            .map(|i| self.entries[i].1.as_str())
    }
}

struct DiskComponentBuilder {
    fill: Block,
    next_id: u64,
    indexes: Vec<String>,
    block_ids: Vec<u64>,
}

impl DiskComponentBuilder {
    fn new(next_id: u64) -> Self {
        Self {
            fill: Block::new(),
            next_id,
            indexes: Vec::new(),
            block_ids: Vec::new(),
        }
    }

    fn add_block(&mut self, block_id: u64, first_key: &str) {
        self.block_ids.push(block_id);
        self.indexes.push(first_key.to_string());
    }

    fn write_fill_block(&mut self) -> Result<(), String> {
        let filled = mem::replace(&mut self.fill, Block::new());
        filled.write_into(&format!("{}/{}.blk", LSM_ROOT, self.next_id))?;
        self.next_id += 1;
        Ok(())
    }

    fn write_entry(&mut self, k: &str, v: &str) -> Result<(), String> {
        self.fill.entries.push((k.to_string(), v.to_string()));
        if self.fill.entries.len() >= MAX_BLOCK_KV_NUM {
            self.write_fill_block()?;
        }
        Ok(())
    }

    fn flush(mut self) -> Result<(Vec<String>, Vec<u64>, u64), String> {
        if !self.fill.entries.is_empty() {
            self.write_fill_block()?;
        }
        Ok((self.indexes, self.block_ids, self.next_id))
    }
}

struct DiskComponent {
    next_id: u64,
    indexes: Vec<String>,
    block_ids: Vec<u64>,
}

impl DiskComponent {
    fn load_block_ids(name: &str) -> Result<(Vec<u64>, Vec<String>), String> {
        let mut block_ids = Vec::new();
        let mut indexes = Vec::new();
        let mut f = Fs::open(name, false)?;
        loop {
            match ioutil::read_u64(&mut f) {
                Ok(id) => {
                    if !block_ids.is_empty() {
                        indexes.push(Block::load_first_key(&format!("{}/{}.blk", LSM_ROOT, id))?);
                    }
                    block_ids.push(id);
                }
                Err(e) if e == "EOF" => break,
                Err(e) => return Err(e),
            };
        }
        Ok((block_ids, indexes))
    }

    fn store_block_ids(name: &str, block_ids: &[u64]) -> Result<(), String> {
        let mut d = Vec::new();
        for &id in block_ids {
            ioutil::write_u64(&mut d, id);
        }
        let mut f = Fs::create(name, true)?;
        f.write(&d);
        Ok(())
    }

    fn load() -> Result<Self, String> {
        let metadata = format!("{}/metadata", LSM_ROOT);
        if !Fs::exist(&metadata) {
            Fs::create(&metadata, true)?;
        }
        let (block_ids, indexes) = Self::load_block_ids(&metadata)?;
        let next_id = block_ids.iter().max().map(|&x| x + 1).unwrap_or(0);
        Ok(Self {
            next_id,
            indexes,
            block_ids,
        })
    }

    fn compact_mem(&mut self, c0: &BTreeMap<String, Option<String>>) -> Result<(), String> {
        if c0.is_empty() {
            return Ok(());
        }

        let mut b = DiskComponentBuilder::new(self.next_id);
        let mut c0_it = c0.iter();
        let mut c0_cur = c0_it.next();

        let mut i = 0usize;
        if let Some((first, _)) = c0_cur {
            while i < self.indexes.len() && first.cmp(&self.indexes[i]) != Ordering::Less {
                b.add_block(self.block_ids[i], if i > 0 { &self.indexes[i] } else { "" });
                i += 1;
            }
        }

        while i < self.block_ids.len() {
            let empty = Block::load(&format!("{}/{}.blk", LSM_ROOT, self.block_ids[i]))?;
            let mut j = 0usize;
            while let Some(c) = c0_cur {
                while j < empty.entries.len() {
                    let e = &empty.entries[j];
                    match e.0.cmp(c.0) {
                        Ordering::Less => b.write_entry(&e.0, &e.1)?,
                        Ordering::Equal => {}
                        Ordering::Greater => break,
                    }
                    j += 1;
                }
                if j >= empty.entries.len() {
                    break;
                }
                if let (k, Some(v)) = c {
                    b.write_entry(k, v)?;
                }
                c0_cur = c0_it.next();
            }
            i += 1;
        }

        while let Some(e) = c0_cur {
            if let (k, Some(v)) = e {
                b.write_entry(k, v)?;
            }
            c0_cur = c0_it.next();
        }

        let (indexes, block_ids, next_id) = b.flush()?;

        Self::store_block_ids(&format!("{}/metadata", LSM_ROOT), &block_ids)?;

        self.block_ids = block_ids;
        self.indexes = indexes;
        self.next_id = next_id;

        Ok(())
    }

    fn search(&self, k: &str) -> Result<Option<String>, String> {
        if self.block_ids.is_empty() {
            return Ok(None);
        }

        let i = self
            .indexes
            .binary_search_by(|probe| probe.as_str().cmp(k))
            .map(|i| i + 1)
            .unwrap_or_else(|e| e);

        Block::load(&format!("{}/{}.blk", LSM_ROOT, self.block_ids[i]))
            .map(|blk| blk.search(k).map(|s| s.to_string()))
    }
}

pub struct LsmTree {
    c0: BTreeMap<String, Option<String>>,
    c1: DiskComponent,
}

impl LsmTree {
    pub fn load() -> Result<Self, String> {
        Ok(Self {
            c0: BTreeMap::new(),
            c1: DiskComponent::load()?,
        })
    }

    fn insert_kv(&mut self, k: String, v: Option<String>) -> Result<bool, String> {
        if self.c0.len() >= MAX_MEM_KV_NUM {
            self.c1.compact_mem(&self.c0)?;
            self.c0.clear();
        }
        Ok(self.c0.insert(k, v).is_some())
    }

    pub fn insert(&mut self, k: String, v: String) -> Result<bool, String> {
        self.insert_kv(k, Some(v))
    }

    pub fn remove(&mut self, k: &str) -> Result<bool, String> {
        self.insert_kv(k.to_string(), None)
    }

    pub fn search(&self, k: &str) -> Result<Option<String>, String> {
        if let Some(v) = self.c0.get(k) {
            return Ok(v.as_ref().map(|v| v.clone()));
        }
        self.c1.search(k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_table() {
        let mut tree = LsmTree::load().expect("load failed");
        tree.insert("a".to_string(), "0".to_string()).unwrap();
        tree.insert("b".to_string(), "1".to_string()).unwrap();

        assert_eq!(tree.search("a").unwrap(), Some("0".to_string()));
        assert_eq!(tree.search("c").unwrap(), None);
    }
}
