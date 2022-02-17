use log::debug;
use std::{
    collections::HashSet,
    hash::Hash,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
};

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

/// Returns solution and number of iterations
pub fn dfs_parallel<G>(graph: G, start: G::Node) -> Result<(G::Node, usize), (String, usize)>
where
    G: Graph + Clone + Send + 'static,
    G::Node: Send + 'static,
{
    let iterations = Arc::new(AtomicUsize::new(0));
    let cpus = num_cpus::get();
    let queue = Arc::new(Mutex::new(vec![start]));
    let current_tasks = Arc::new(AtomicUsize::new(1)); // 1 because we added `start`
    let visited = Arc::new(Mutex::new(HashSet::new()));
    let finished = Arc::new(AtomicBool::new(false));

    let (s, r) = mpsc::channel();

    crossbeam::scope(|scope| {
        for i in 0..cpus {
            let queue = queue.clone();
            let s = s.clone();
            let graph = graph.clone();
            let visited = visited.clone();
            let iterations = iterations.clone();
            let current_tasks = current_tasks.clone();
            let finished = finished.clone();
            scope.spawn(move |_| {
                debug!("[Handler {i}] Started");
                loop {
                    if finished.load(Ordering::SeqCst) {
                        break;
                    }
                    let msg = queue.lock().unwrap().pop();
                    if let Some(mut node) = msg {
                        debug!("[Handler {i}] Task received");
                        iterations.fetch_add(1, Ordering::SeqCst);
                        match graph.check_goal(&mut node) {
                            GraphControl::Finish => {
                                debug!("[Handler {i}] Sending FINISH event");
                                finished.fetch_or(true, Ordering::SeqCst);
                                let i = iterations.load(Ordering::SeqCst);
                                s.send(Ok((node, i))).unwrap();
                                break;
                            }
                            GraphControl::Prune => {}
                            GraphControl::Continue => {
                                for neighbour in graph.neighbours(&node) {
                                    if visited.lock().unwrap().contains(&neighbour) {
                                        continue;
                                    }
                                    debug!("[Handler {i}] Queueing discovered neighbour");
                                    queue.lock().unwrap().push(neighbour);
                                    current_tasks.fetch_add(1, Ordering::SeqCst);
                                }
                            }
                        }
                        visited.lock().unwrap().insert(node);
                        current_tasks.fetch_sub(1, Ordering::SeqCst);
                    } else if current_tasks.load(Ordering::SeqCst) == 0 {
                        debug!("[Handler {i}] current_tasks==0, stopping the solver...");
                        finished.fetch_or(true, Ordering::SeqCst);
                        let i = iterations.load(Ordering::SeqCst);
                        s.send(Err(("No solution found :C".to_string(), i)))
                            .unwrap();
                        break;
                    }
                }
            });
        }
    })
    .unwrap();

    r.recv().unwrap()
}
