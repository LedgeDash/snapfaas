//! The SnapFaaS Controller
//!
//! The Controller consists of a request manager (file or HTTP) and a pool of workers.
//! The gateway takes in requests. The controller assigns each request a worker.
//! Each worker is responsible for finding a VM to handle the request and proxies the response.
//!
//! The Controller maintains several states:
//!   1. kernel path
//!   2. kernel boot argument
//!   3. function store and their files' locations

use std::string::String;
use log::{error, warn, info};
use url::{Url, ParseError};
use clap::{Arg, App};
use simple_logger;
use futures::{Future, Async, Poll};
use gateway::Gateway;
use snapfaas::*;
use std::thread;

mod configs;
mod gateway;

fn main() {

    simple_logger::init().expect("simple_logger init failed");

    let matches = App::new("SnapFaaS controller")
                          .version("1.0")
                          .author("David H. Liu <hao.liu@princeton.edu>")
                          .about("Launch and configure SnapFaaS controller")
                          .arg(Arg::with_name("config")
                               .short("c")
                               .long("config")
                               .takes_value(true)
                               .help("Controller config YAML file"))
                          .arg(Arg::with_name("kernel")
                               .long("kernel")
                               .takes_value(true)
                               .help("URL to the kernel binary"))
                          .arg(Arg::with_name("kernel boot args")
                               .long("kernel_args")
                               .takes_value(true)
                               .default_value("quiet console=none reboot=k panic=1 pci=off")
                               .help("Default kernel boot argument"))
                          .arg(Arg::with_name("requests file")
                               .long("requests_file")
                               .takes_value(true)
                               .help("File containing JSON-lines of requests"))
                          .arg(Arg::with_name("port number")
                               .long("port")
                               .short("p")
                               .takes_value(true)
                               .help("Port on which SnapFaaS accepts requests"))
                          .get_matches();

    // populate the in-memory config struct
    let mut ctr_config = configs::ControllerConfig::new(matches.value_of("config"));

    if let Some(kernel_url) = matches.value_of("kernel") {
        ctr_config.set_kernel_path(kernel_url);
    };

    if let Some(kernel_boot_args) = matches.value_of("kernel boot args") {
        ctr_config.set_kernel_boot_args(kernel_boot_args);
    };

    info!("{:?}", ctr_config);

    // prepare worker pool
    let mut wp = workerpool::WorkerPool::new();

    // start gateway
    // TODO:support an HTTP gateway in addition to file gateway
    let request_file_url = matches.value_of("requests file").expect("rf");
    let gateway = gateway::FileGateway::listen(request_file_url).unwrap();

    // start admitting and processing incoming requests
    for req in gateway.incoming() {
        // ignore invalid requests for now
        if req.is_err() {
            continue;
        }
        println!("req (main): {:?}", req);

        // acquire a worker from the pool and just pass it the request
        // send_req() here does not block
        let worker = wp.acquire();
        worker.send_req(req.unwrap());

        // it's passible for worker to be deallocated when exiting this for loop
    }

    thread::sleep(std::time::Duration::from_secs(2))
}

