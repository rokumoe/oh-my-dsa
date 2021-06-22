#![allow(unused)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Once};

pub enum Whence {
    Cur(isize),
    Set(isize),
    End(isize),
}

struct FileInner {
    data: Vec<u8>,
}

pub struct File {
    inner: Arc<Mutex<FileInner>>,
    p: usize,
    append_only: bool,
}

impl File {
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let f = self.inner.lock().unwrap();
        self.p = self.p.min(f.data.len());
        let n = buf.len().min(f.data.len() - self.p);
        if n > 0 {
            buf[..n].copy_from_slice(&f.data[self.p..self.p + n]);
            self.p += n;
        }
        n
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), String> {
        let n = self.read(buf);
        if n == buf.len() {
            Ok(())
        } else {
            if n == 0 {
                Err("EOF".to_string())
            } else {
                Err("unexpected EOF".to_string())
            }
        }
    }

    pub fn append(&self, buf: &[u8]) {
        self.inner.lock().unwrap().data.extend_from_slice(buf);
    }

    pub fn write(&mut self, buf: &[u8]) {
        if self.append_only {
            self.append(buf);
        } else {
            let mut f = self.inner.lock().unwrap();
            self.p = self.p.min(f.data.len());
            let n = buf.len().min(f.data.len() - self.p);
            if n > 0 {
                f.data[self.p..].copy_from_slice(buf);
            }
            let m = buf.len() - n;
            if m > 0 {
                f.data.extend_from_slice(&buf[n..]);
            }
            self.p += buf.len();
        }
    }

    pub fn seek(&mut self, whence: Whence) -> usize {
        let f = self.inner.lock().unwrap();
        let mut p = self.p.min(f.data.len()) as isize;
        match whence {
            Whence::Cur(offs) => p = p + offs,
            Whence::Set(offs) => p = offs,
            Whence::End(offs) => p = f.data.len() as isize + offs,
        }
        if p < 0 {
            p = 0;
        } else if p > f.data.len() as isize {
            p = f.data.len() as isize;
        }
        self.p = p as usize;
        self.p
    }

    pub fn truncate(&mut self, n: usize) {
        let mut f = self.inner.lock().unwrap();
        let m = f.data.len().min(n);
        f.data.truncate(m);
        self.p = m;
    }

    pub fn size(&self) -> usize {
        self.inner.lock().unwrap().data.len()
    }
}

pub struct FsImpl {
    files: BTreeMap<String, Arc<Mutex<FileInner>>>,
}

impl FsImpl {
    pub fn new() -> Self {
        FsImpl {
            files: BTreeMap::new(),
        }
    }

    pub fn open(&self, name: &str, append_only: bool) -> Result<File, String> {
        self.files
            .get(name)
            .map(|f| File {
                inner: f.clone(),
                p: 0,
                append_only,
            })
            .ok_or("no such file".to_string())
    }

    pub fn create(&mut self, name: &str, overwrite: bool) -> Result<File, String> {
        (!self.files.contains_key(name) || overwrite)
            .then(|| {
                let data = Arc::new(Mutex::new(FileInner { data: Vec::new() }));
                self.files.insert(name.to_string(), data.clone());
                File {
                    inner: data,
                    p: 0,
                    append_only: false,
                }
            })
            .ok_or("file existed".to_string())
    }

    pub fn exist(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    pub fn ls(&self, path: &str, rec: bool) -> Vec<String> {
        let mut ents = self
            .files
            .keys()
            .filter(|name| name.starts_with(&path))
            .filter_map(|name| {
                rec.then(|| name.clone()).or_else(|| {
                    name[path.len()..]
                        .find('/')
                        .map(|p| name[..path.len() + p + 1].to_string())
                        .or_else(|| Some(name.clone()))
                })
            })
            .collect::<Vec<_>>();
        if !rec {
            ents.sort();
            ents.dedup();
        }
        ents
    }

    pub fn rm(&mut self, name: &str) {
        self.files.remove(name);
    }
}

pub struct Fs;

impl Fs {
    fn shared() -> &'static Mutex<FsImpl> {
        static mut FS: Option<Mutex<FsImpl>> = None;
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe { FS = Some(Mutex::new(FsImpl::new())) });
        unsafe { FS.as_ref().unwrap() }
    }

    pub fn open(name: &str, append_only: bool) -> Result<File, String> {
        Self::shared().lock().unwrap().open(name, append_only)
    }

    pub fn create(name: &str, overwrite: bool) -> Result<File, String> {
        Self::shared().lock().unwrap().create(name, overwrite)
    }

    pub fn exist(name: &str) -> bool {
        Self::shared().lock().unwrap().exist(name)
    }

    pub fn ls(path: &str, rec: bool) -> Vec<String> {
        Self::shared().lock().unwrap().ls(path, rec)
    }

    pub fn join(a: &str, b: &str) -> String {
        let mut p = String::with_capacity(a.len() + b.len() + 1);
        p.push_str(a.trim_end_matches('/'));
        p.push('/');
        p.push_str(b.trim_start_matches('/'));
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_open() {
        let mut fs = FsImpl::new();
        {
            fs.create("hello.txt", false).unwrap();
            assert!(fs.create("hello.txt", false).is_err());
            assert!(fs.create("hello.txt", true).is_ok());
        }
        assert!(fs.open("hello.txt", false).is_ok());
        assert!(fs.open("hello.log", false).is_err());
    }

    #[test]
    fn test_read_write() {
        let mut fs = FsImpl::new();
        let mut f = fs.create("hello.txt", false).unwrap();
        f.write(b"hello");
        f.seek(Whence::Set(0));
        let mut rd = vec![0; 5];
        assert_eq!(f.read(&mut rd), 5);
        assert_eq!(&rd, b"hello");
        f.seek(Whence::Set(0));
        f.truncate(0);
        f.write(b"hola");
        f.seek(Whence::Set(0));
        assert_eq!(f.read(&mut rd), 4);
        assert_eq!(&rd[..4], b"hola");
    }

    #[test]
    fn test_append() {
        let mut fs = FsImpl::new();
        {
            let mut f = fs.create("hello.txt", false).unwrap();
            f.write(b"hello");
        }
        let mut f2 = fs.open("hello.txt", true).unwrap();
        f2.write(b"world");
        f2.seek(Whence::Set(0));
        let mut rd = vec![0; 10];
        assert_eq!(f2.read(&mut rd), 10);
        assert_eq!(&rd, b"helloworld");
    }

    #[test]
    fn test_ls() {
        let mut fs = FsImpl::new();
        fs.create("/a", false).unwrap();
        fs.create("/a/a.txt", false).unwrap();
        fs.create("/a/b.txt", false).unwrap();
        fs.create("/b/hello.txt", false).unwrap();
        assert_eq!(fs.ls("/", false).len(), 3);
        assert_eq!(fs.ls("/a/", false).len(), 2);
        assert_eq!(fs.ls("/", true).len(), 4);
    }
}

pub mod ioutil {
    use super::File;

    pub fn read_u8(f: &mut File) -> Result<u8, String> {
        let mut fixed = [0u8; 1];
        f.read_exact(&mut fixed[..])?;
        Ok(fixed[0])
    }

    pub fn read_u32(f: &mut File) -> Result<u32, String> {
        let mut fixed = [0u8; 4];
        f.read_exact(&mut fixed[..])?;
        Ok(u32::from_le_bytes(fixed))
    }

    pub fn read_u64(f: &mut File) -> Result<u64, String> {
        let mut fixed = [0u8; 8];
        f.read_exact(&mut fixed[..])?;
        Ok(u64::from_le_bytes(fixed))
    }

    pub fn read_string(f: &mut File) -> Result<String, String> {
        let n = read_u32(f)? as usize;
        let mut buf = vec![0u8; n];
        f.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| e.to_string())
    }

    pub fn read_vec<T, R>(f: &mut File, r: R) -> Result<Vec<T>, String>
    where
        R: Fn(&mut File) -> Result<T, String>,
    {
        let n = read_u32(f)? as usize;
        let mut a = Vec::with_capacity(n);
        for _ in 0..n {
            a.push(r(f)?);
        }
        Ok(a)
    }

    pub fn write_u8(d: &mut Vec<u8>, x: u8) {
        d.push(x);
    }

    pub fn write_u32(d: &mut Vec<u8>, x: u32) {
        let buf = x.to_le_bytes();
        d.extend_from_slice(&buf[..]);
    }

    pub fn write_u64(d: &mut Vec<u8>, x: u64) {
        let buf = x.to_le_bytes();
        d.extend_from_slice(&buf[..]);
    }

    pub fn write_str(d: &mut Vec<u8>, s: &str) {
        write_u32(d, s.len() as u32);
        d.extend_from_slice(s.as_bytes());
    }

    pub fn write_slice<T, W>(d: &mut Vec<u8>, a: &[T], w: W)
    where
        W: Fn(&mut Vec<u8>, &T),
    {
        let n = (a.len() as u32).to_le_bytes();
        d.extend_from_slice(&n[..]);
        for x in a {
            w(d, x);
        }
    }

    pub fn read_buf_u32(d: &[u8]) -> u32 {
        let mut nbuf = [0u8; 4];
        nbuf.copy_from_slice(&d[..4]);
        u32::from_le_bytes(nbuf)
    }

    pub fn read_buf_str(d: &[u8]) -> &str {
        let n = read_buf_u32(d) as usize;
        ::std::str::from_utf8(&d[4..4 + n]).unwrap()
    }
}
