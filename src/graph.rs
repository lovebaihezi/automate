use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct Graph<N, V>
where
    N: Hash + Eq,
    V: Hash + Eq,
{
    map: HashMap<(N, V), HashSet<N>>,
}

impl<N, V> Graph<N, V>
where
    N: Hash + Eq,
    V: Hash + Eq,
{
    pub fn with_capaticy(edge_amount: usize) -> Self {
        Self {
            map: HashMap::with_capacity(edge_amount),
        }
    }
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn add_edge(&mut self, from: (N, V), to: N) {
        if let Some(node) = self.map.get_mut(&from) {
            node.insert(to);
        } else {
            let mut set = HashSet::new();
            set.insert(to);
            self.map.insert(from, set);
        }
    }
    pub fn next_state(&self, state: N, v: V) -> Option<&HashSet<N>> {
        self.map.get(&(state, v))
    }
}

#[macro_export]
macro_rules! Graph {
    { $($from:expr =>($v:expr)=> $to:expr,)* } => {
        {
            let mut graph = $crate::graph::Graph::new();
            $(graph.add_edge(($from, $v), $to);)*
            graph
        }
    }
}

#[cfg(test)]
mod graph {
    #[test]
    fn macro_expand() {
        let graph = Graph! {
            1 => ('a') => 3,
            1 => ('b') => 4,
            2 => ('c') => 5,
        };
    }
}
