//! A fixed size pool (maybe slightly below max, max being total memory/120MB)
//! Acquire a free worker from a pool. This should always succeed because we
//! should not run out of worker threads.
//! A worker takes a reqeust and finds a VM to execute it. 

use crate::worker::Worker;
use std::thread::JoinHandle;
use std::sync::Mutex;
use std::sync::Arc;

const DEFAULT_NUM_WORKERS: usize = 10;

pub struct WorkerPool {
    pool: Arc<Mutex<Vec<Worker>>>,
    handles: Vec<JoinHandle<()>>,
    max_num_workers: usize,
    num_free: usize,
}

impl WorkerPool {
    pub fn new() -> WorkerPool {
        let mut pool = Arc::new(Mutex::new(Vec::with_capacity(DEFAULT_NUM_WORKERS)));
        let mut handles = Vec::with_capacity(DEFAULT_NUM_WORKERS);

        for _ in 0..DEFAULT_NUM_WORKERS {
            let (w, h) = Worker::new(pool.clone());
            handles.push(h);
            pool.lock().unwrap().push(w);
        }

        WorkerPool {
            pool: pool,
            handles: handles,
            max_num_workers: DEFAULT_NUM_WORKERS,
            num_free: DEFAULT_NUM_WORKERS,
        }
    }

    pub fn acquire(&mut self) -> Worker {
        return self.pool.lock().unwrap().pop().expect("Worker pool is empty");
    }
}
