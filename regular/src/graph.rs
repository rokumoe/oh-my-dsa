// https://oi-wiki.org/graph/concept/

use std::collections::VecDeque;

pub mod sc;
pub mod topo;

pub struct Graph {
    pub(self) vs: usize,
    pub(self) adj: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new(vs: usize) -> Self {
        Self {
            vs,
            adj: vec![Vec::new(); vs],
        }
    }

    pub fn add_edge(&mut self, v: usize, w: usize) {
        self.adj[v].push(w);
    }

    pub fn in_degs(&self) -> Vec<usize> {
        let mut in_degs = vec![0usize; self.vs];
        for v in 0..self.vs {
            for &w in self.adj[v].iter() {
                in_degs[w] += 1;
            }
        }
        in_degs
    }

    pub fn print(&self) {
        println!("graph: vs={}", self.vs);
        for v in 0..self.vs {
            println!("| adj vertex {}:", v);
            for &w in self.adj[v].iter() {
                println!("- {} -> {}", v, w);
            }
        }
    }

    pub fn transpose(&self) -> Self {
        let mut g = Self::new(self.vs);
        for v in 0..self.vs {
            for &w in self.adj[v].iter() {
                g.add_edge(w, v);
            }
        }
        g
    }
}

pub fn dfs_recur(g: &Graph, v: usize, visited: &mut [bool], f: &mut impl FnMut(usize)) {
    visited[v] = true;
    f(v);
    for &w in g.adj[v].iter() {
        if visited[w] {
            continue;
        }
        dfs_recur(g, w, visited, f);
    }
}

pub fn dfs(g: &Graph, v: usize, mut f: impl FnMut(usize)) {
    let mut visited = vec![false; g.vs];
    dfs_recur(g, v, &mut visited, &mut f);
}

pub fn bfs(g: &Graph, v: usize, mut f: impl FnMut(usize)) {
    let mut visited = vec![false; g.vs];
    let mut q = VecDeque::new();
    q.push_back(v);
    while let Some(v) = q.pop_front() {
        visited[v] = true;
        f(v);
        for &w in g.adj[v].iter() {
            if !visited[w] {
                q.push_back(w);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Graph;

    #[test]
    fn test_print() {
        let mut g = Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(0, 4);
        g.add_edge(1, 2);
        g.add_edge(1, 3);
        g.add_edge(1, 4);
        g.add_edge(2, 3);
        g.add_edge(3, 4);
        g.print();
    }

    #[test]
    fn test_transpose() {
        let mut g = Graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(0, 4);
        g.add_edge(1, 2);
        g.add_edge(1, 3);
        g.add_edge(1, 4);
        g.add_edge(2, 3);
        g.add_edge(3, 4);
        g.print();
        g.transpose().print();
    }
}
