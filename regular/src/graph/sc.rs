// https://oi-wiki.org/graph/scc/
// https://www.geeksforgeeks.org/tarjan-algorithm-find-strongly-connected-components

use super::*;

pub fn is_sc(g: &Graph) -> bool {
    let mut visited = vec![false; g.vs];
    dfs_recur(g, 0, &mut visited, &mut |_| {});
    if !visited.into_iter().all(|x| x) {
        return false;
    }
    let mut visited = vec![false; g.vs];
    let r = g.transpose();
    dfs_recur(&r, 0, &mut visited, &mut |_| {});
    visited.into_iter().all(|x| x)
}

struct TarjanContext<'a> {
    g: &'a Graph,
    dfn_cnt: usize,
    dfn: Vec<usize>, // dfs 时顶点 u 被搜索的编号
    low: Vec<usize>, // 顶点 u 回溯到的最早编号
    stack: Vec<usize>,
    in_stack: Vec<bool>,
    scc: Vec<Vec<usize>>,
}

fn tarjan_scc_recur(ctx: &mut TarjanContext, u: usize) {
    ctx.dfn_cnt += 1;
    ctx.dfn[u] = ctx.dfn_cnt;
    ctx.low[u] = ctx.dfn_cnt;
    ctx.stack.push(u);
    ctx.in_stack[u] = true;
    for &v in ctx.g.adj[u].iter() {
        if ctx.dfn[v] == 0 {
            tarjan_scc_recur(ctx, v);
            ctx.low[u] = ctx.low[u].min(ctx.low[v]);
        } else if ctx.in_stack[v] {
            ctx.low[u] = ctx.low[u].min(ctx.dfn[v]);
        }
    }
    if ctx.dfn[u] == ctx.low[u] {
        let mut scc = Vec::new();
        while let Some(&top) = ctx.stack.last() {
            ctx.stack.pop();
            ctx.in_stack[top] = false;
            scc.push(top);
            if top == u {
                break;
            }
        }
        ctx.scc.push(scc);
    }
}

pub fn tarjan_scc(g: &Graph) -> Vec<Vec<usize>> {
    let mut ctx = TarjanContext {
        g,
        dfn: vec![0; g.vs],
        low: vec![0; g.vs],
        dfn_cnt: 0,
        stack: Vec::new(),
        in_stack: vec![false; g.vs],
        scc: Vec::new(),
    };
    for u in 0..g.vs {
        if ctx.dfn[u] != 0 {
            continue;
        }
        tarjan_scc_recur(&mut ctx, u);
    }
    ctx.scc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sc() {
        let mut g1 = Graph::new(5);
        g1.add_edge(0, 1);
        g1.add_edge(1, 2);
        g1.add_edge(2, 3);
        g1.add_edge(3, 0);
        g1.add_edge(2, 4);
        g1.add_edge(4, 2);
        println!("g1 is_sc {}", is_sc(&g1));
        let mut g2 = Graph::new(4);
        g2.add_edge(0, 1);
        g2.add_edge(1, 2);
        g2.add_edge(2, 3);
        println!("g2 is_sc {}", is_sc(&g2));
    }

    #[test]
    fn test_tarjan_scc() {
        let mut g1 = Graph::new(5);
        g1.add_edge(0, 1);
        g1.add_edge(1, 2);
        g1.add_edge(2, 3);
        g1.add_edge(3, 0);
        g1.add_edge(2, 4);
        let scc = tarjan_scc(&g1);
        println!("scc = {:?}", scc);
    }
}
