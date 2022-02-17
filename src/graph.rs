use std::{collections::HashSet, hash::Hash};

pub trait Graph {
    type Node: Clone + Eq + Hash;
    fn neighbours(&self, node: &Self::Node) -> Vec<Self::Node>;
    fn check_goal(&self, node: &mut Self::Node) -> GraphControl;
}

pub enum GraphControl {
    Finish,
    Continue,
    Prune,
}

/// Returns solution and number of iterations
pub fn dfs<G: Graph>(graph: G, start: G::Node) -> Result<(G::Node, usize), (String, usize)> {
    let mut iterations = 0;
    let mut queue = Vec::new();
    let mut visited = HashSet::new();
    queue.push(start);

    while let Some(mut node) = queue.pop() {
        iterations += 1;
        match graph.check_goal(&mut node) {
            GraphControl::Finish => {
                return Ok((node, iterations));
            }
            GraphControl::Prune => {}
            GraphControl::Continue => {
                for neighbour in graph.neighbours(&node) {
                    if visited.contains(&neighbour) {
                        continue;
                    }
                    queue.push(neighbour);
                }
            }
        }
        visited.insert(node);
    }
    Err(("No solution found :C".to_string(), iterations))
}
