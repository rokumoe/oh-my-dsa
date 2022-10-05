// https://oi-wiki.org/graph/topo/

use super::*;

pub fn toposort_kahn(g: &Graph) -> Vec<usize> {
    let mut in_degs = g.in_degs();
    let mut q = VecDeque::new();
    in_degs
        .iter()
        .enumerate()
        .filter_map(|(v, &in_deg)| if in_deg == 0 { Some(v) } else { None })
        .for_each(|v| q.push_back(v));
    let mut topo = Vec::new();
    while let Some(u) = q.pop_front() {
        topo.push(u);
        for &v in g.adj[u].iter() {
            in_degs[v] -= 1;
            if in_degs[v] == 0 {
                q.push_back(v);
            }
        }
    }
    topo
}

fn toposort_dfs_recur(topo: &mut Vec<usize>, g: &Graph, visited: &mut [i32], u: usize) -> bool {
    visited[u] = 1;
    for &v in g.adj[u].iter() {
        if visited[v] == 1 {
            return false;
        }
        if visited[v] != 0 {
            continue;
        }
        if !toposort_dfs_recur(topo, g, visited, v) {
            return false;
        }
    }
    visited[u] = 2;
    topo.push(u);
    true
}

pub fn toposort_dfs(g: &Graph) -> Option<Vec<usize>> {
    let mut topo = Vec::new();
    let mut visited = vec![0; g.vs];
    for u in 0..g.vs {
        if visited[u] != 0 {
            continue;
        }
        if !toposort_dfs_recur(&mut topo, g, &mut visited, u) {
            return None;
        }
    }
    topo.reverse();
    Some(topo)
}

#[cfg(test)]
mod tests {
    use crate::debug::set_debug;
    use crate::graph::topo::toposort_dfs;
    use crate::graph::Graph;

    use super::toposort_kahn;

    fn get_g1() -> Graph {
        let mut g = Graph::new(5);
        g.add_edge(2, 4);
        g.add_edge(0, 3);
        g.add_edge(0, 1);
        g.add_edge(3, 4);
        g.add_edge(1, 2);
        g.add_edge(3, 2);
        g.add_edge(1, 3);
        g
    }

    #[test]
    fn test_toposort_kahn() {
        set_debug(true);
        let g1 = get_g1();
        let topo = toposort_kahn(&g1);
        D!(topo);
    }

    #[test]
    fn test_toposort_dfs() {
        set_debug(true);
        let g1 = get_g1();
        let topo = toposort_dfs(&g1);
        D!(topo);
    }
}
