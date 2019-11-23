//! Workers proxies requests and responses between the request manager and VMs.
//! Each worker runs in its own thread and is modeled as the following state
//! machine:
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver, SendError};
use crate::request::Request;
use std::time::Duration;

/// From JoinHandle we can get the &Thread which then gives us ThreadId and
/// park() function. We can't peel off the JoinHandle to get Thread because
/// JoinHandle struct owns Thread as a field.
#[derive(Debug,Clone)]
pub struct Worker {
    req_sender: Sender<Request>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    WaitForReq,
    Done
}

impl Worker {
    pub fn new(pool: Arc<Mutex<Vec<Worker>>>) -> (Worker, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let worker = Worker {req_sender: tx};
        let w = worker.clone();

        let handle = thread::spawn(move || {
            loop {
                let state = State::WaitForReq;
                let req = rx.recv();
                println!("req (worker): {:?}", req);
                pool.lock().unwrap().push(w.clone());
            }
            
        });

        return (worker, handle);
    }

    /*
    pub fn transition(&mut self, s: State) {
        self.state = s;
    }

    fn wait_for_req(rx: Receiver<Request>) {
        let req = rx.recv();

    }

    fn echo_req(req: &Request) {
        println!("req (worker): {:?}", req);
    }
    */

    pub fn send_req(self, req: Request) -> Result<(), SendError<Request>> {
        return self.req_sender.send(req);
    }
}
