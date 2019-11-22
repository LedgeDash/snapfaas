//! A fixed size pool (maybe slightly below max, max being total memory/120MB)
//! Acquire a free worker from a pool. This should always succeed because we
//! should not run out of worker threads.
//! A worker takes a reqeust and finds a VM to execute it. 

use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Sender, Receiver, SendError};
use crate::worker::Worker;
use crate::request::Request;

const DEFAULT_NUM_WORKERS: usize = 10;

pub struct WorkerPool {
    pool: Vec<Worker>,
    max_num_workers: usize,
    req_sender: Sender<Request>
}

impl WorkerPool {
    pub fn new() -> WorkerPool {
        let mut pool = Vec::with_capacity(DEFAULT_NUM_WORKERS);

        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));


        for _ in 0..DEFAULT_NUM_WORKERS {
            pool.push(Worker::new(rx.clone()));
        }

        WorkerPool {
            pool: pool,
            max_num_workers: DEFAULT_NUM_WORKERS,
            req_sender: tx,
        }
    }

    pub fn send_req(&self, req: Request) {
        self.req_sender.send(req);
    }

    
}
