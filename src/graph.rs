use std::{
    collections::HashSet,
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
};

use crossbeam::channel::unbounded;
use itertools::Itertools;
use log::debug;

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
    let queue = {
        let mut queue = Vec::new();
        queue.push(start);
        Arc::new(Mutex::new(queue))
    };
    let visited = Arc::new(Mutex::new(HashSet::new()));

    let (s_queue, r_queue) = unbounded();
    let (s_tasks, r_tasks) = unbounded();
    let (s, r) = unbounded();

    let _queue_handler = {
        let queue = queue.clone();
        let r_queue = r_queue.clone();
        let started_pair = started_pair.clone();
        thread::spawn(move || {
            let mut notified = false;
            while let Ok(msg) = r_queue.recv() {
                debug!("[Queue handler] Receiving msg...");
                queue.lock().unwrap().push(msg);
                debug!("[Queue handler] Added msg to queue...");
                if !notified {
                    let (lock, cvar) = &*started_pair;
                    let mut started = lock.lock().unwrap();
                    *started = true;
                    cvar.notify_one();
                    notified = true;
                    debug!("[Queue handler] Notifying started=true");
                }
            }
        })
    };

    let _dispatcher = {
        let queue = queue.clone();
        let s_tasks = s_tasks.clone();
        let s = s.clone();
        let iterations = iterations.clone();
        let free_tasks = free_tasks.clone();
        thread::spawn(move || {
            debug!("[Dispatcher] Waiting to start...");
            let (lock, cvar) = &*started_pair;
            let mut started = lock.lock().unwrap();
            while !*started {
                started = cvar.wait(started).unwrap();
            }
            debug!("[Dispatcher] Started");
            loop {
                if free_tasks.load(Ordering::SeqCst) > 0 {
                    let a = queue.lock().unwrap().pop();
                    if let Some(msg) = a {
                        debug!("[Dispatcher] Dispatching msg.");
                        s_tasks.send(msg).unwrap();
                    } else {
                        if free_tasks.load(Ordering::SeqCst) == cpus {
                            debug!("[Dispatcher] Free tasks and empty queue...");
                            debug!("==> cpus={}", cpus);
                            debug!("==> free_tasks={}", free_tasks.load(Ordering::SeqCst));
                            debug!("==> len(queue)={}", queue.lock().unwrap().len());
                            let i = iterations.load(Ordering::SeqCst);
                            s.send(Err(("No solution found :C".to_string(), i)))
                                .unwrap();
                        }
                    }
                }
            }
        })
    };

    let _tasks = (0..cpus)
        .map(|i| {
            let s = s.clone();
            let r_tasks = r_tasks.clone();
            let s_queue = s_queue.clone();
            let graph = graph.clone();
            let free_tasks = free_tasks.clone();
            let visited = visited.clone();
            let iterations = iterations.clone();
            thread::spawn(move || {
                while let Ok(mut node) = r_tasks.recv() {
                    debug!("[Handler {i}] Task received");
                    iterations.fetch_add(1, Ordering::SeqCst);
                    free_tasks.fetch_add(1, Ordering::SeqCst);
                    match graph.check_goal(&mut node) {
                        GraphControl::Finish => {
                            debug!("[Handler {i}] Sending FINISH...");
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
                                debug!("[Handler {i}] Sending neighbour...");
                                s_queue.send(neighbour).unwrap();
                            }
                        }
                    }
                    visited.lock().unwrap().insert(node);
                }
                free_tasks.fetch_sub(1, Ordering::SeqCst);
            })
        })
        .collect_vec();

    s_queue.send(start).unwrap();
    r.recv().unwrap()
}
