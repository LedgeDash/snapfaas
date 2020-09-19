#[macro_use(crate_version, crate_authors)]
extern crate clap;
/// This binary is used to launch a single instance of firerunner
/// It reads a request from stdin, launches a VM based on cmdline inputs, sends
/// the request to VM, waits for VM's response and finally prints the response
/// to stdout, kills the VM and exits.
use snapfaas::vm::Vm;
use snapfaas::configs::FunctionConfig;
use std::os::unix::net::UnixListener;

use clap::{App, Arg};

const CID: u32 = 124;

fn main() {
    simple_logger::init().expect("simple_logger init failed");
    let cmd_arguments = App::new("fireruner wrapper")
        .version(crate_version!())
        .author(crate_authors!())
        .about("launch a single firerunner vm.")
        .arg(
            Arg::with_name("kernel")
                .short("k")
                .long("kernel")
                .value_name("kernel")
                .takes_value(true)
                .required(true)
                .help("path the the kernel binary")
        )
        .arg(
            Arg::with_name("kernel_args")
                .short("c")
                .long("kernel_args")
                .value_name("kernel_args")
                .takes_value(true)
                .required(false)
                .help("kernel boot args")
        )
        .arg(
            Arg::with_name("rootfs")
                .short("r")
                .long("rootfs")
                .value_name("rootfs")
                .takes_value(true)
                .required(true)
                .help("path to the root file system")
        )
        .arg(
            Arg::with_name("appfs")
                .long("appfs")
                .value_name("appfs")
                .takes_value(true)
                .required(false)
                .help("path to the app file system")
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .help("microvm unique identifier")
                .default_value("1234")
                .required(true)
                .takes_value(true)
        )
        .arg(
            Arg::with_name("load_dir")
                .long("load_dir")
                .takes_value(true)
                .required(false)
                .help("if specified start vm from a snapshot under the given directory")
        )
        .arg(
            Arg::with_name("dump_dir")
                .long("dump_dir")
                .takes_value(true)
                .required(false)
                .help("if specified creates a snapshot right after runtime is up under the given directory")
        )
        .arg(
            Arg::with_name("mem_size")
                 .long("mem_size")
                 .value_name("MEMSIZE")
                 .takes_value(true)
                 .required(true)
                 .help("Guest memory size in MB (default is 128)")
        )
        .arg(
            Arg::with_name("vcpu_count")
                 .long("vcpu_count")
                 .value_name("VCPUCOUNT")
                 .takes_value(true)
                 .required(true)
                 .help("Number of vcpus (default is 1)")
        )
        .arg(
            Arg::with_name("copy_base_memory")
                 .long("copy_base")
                 .value_name("COPYBASE")
                 .takes_value(false)
                 .required(false)
                 .help("Restore base snapshot memory by copying")
        )
        .arg(
            Arg::with_name("diff_dirs")
                 .long("diff_dirs")
                 .value_name("DIFFDIRS")
                 .takes_value(true)
                 .required(false)
                 .help("Comma-separated list of diff snapshots")
        )
        .arg(
            Arg::with_name("copy_diff_memory")
                 .long("copy_diff")
                 .value_name("COPYDIFF")
                 .takes_value(false)
                 .required(false)
                 .help("If a diff snapshot is provided, restore its memory by copying")
        )
        .arg(
            Arg::with_name("network")
                .long("network")
                .value_name("NETWORK")
                .takes_value(true)
                .required(false)
                .help("newtork device of format TAP_NAME/MAC_ADDRESS")
        )
        .arg(
            Arg::with_name("firerunner")
                .long("firerunner")
                .value_name("FIRERUNNER")
                .takes_value(true)
                .required(true)
                .default_value("target/release/firerunner")
                .help("path to the firerunner binary")
        )
        .arg(
            Arg::with_name("force exit")
                .long("force_exit")
                .value_name("FORCEEXIT")
                .takes_value(false)
                .required(false)
                .help("force fc_wrapper to exit once firerunner exits")
        )
        .arg(
            // by default base snapshot is not opened with O_DIRECT
            Arg::with_name("odirect base")
                .long("odirect_base")
                .value_name("ODIRECT_BASE")
                .takes_value(false)
                .required(false)
                .help("If present, open base snapshot's memory file with O_DIRECT")
        )
        .arg(
            // by default diff snapshot is opened with O_DIRECT
            Arg::with_name("no odirect diff")
                .long("no_odirect_diff")
                .value_name("NO_ODIRECT_DIFF")
                .takes_value(false)
                .required(false)
                .help("If present, open diff snapshot's memory file without O_DIRECT")
        )
        .arg(
            Arg::with_name("no odirect rootfs")
                .long("no_odirect_root")
                .value_name("NO_ODIRECT_ROOT")
                .takes_value(false)
                .required(false)
                .help("If present, open rootfs file without O_DIRECT")
        )
        .arg(
            Arg::with_name("no odirect appfs")
                .long("no_odirect_app")
                .value_name("NO_ODIRECT_APP")
                .takes_value(false)
                .required(false)
                .help("If present, open appfs file without O_DIRECT")
        )
        .get_matches();

    // Create a FunctionConfig value based on cmdline inputs
    let vm_app_config = FunctionConfig {
        name: "app".to_string(), //dummy value
        runtimefs: cmd_arguments.value_of("rootfs").expect("rootfs").to_string(),
        appfs: cmd_arguments.value_of("appfs").unwrap_or_default().to_string(),
        vcpus: cmd_arguments.value_of("vcpu_count").expect("vcpu")
                            .parse::<u64>().expect("vcpu not int"),
        memory: cmd_arguments.value_of("mem_size").expect("mem_size")
                            .parse::<usize>().expect("mem_size not int"),
        concurrency_limit: 1,
        load_dir: cmd_arguments.value_of("load_dir").map(|s| s.to_string()),
        dump_dir: cmd_arguments.value_of("dump_dir").map(|s| s.to_string()),
        diff_dirs: cmd_arguments.value_of("diff_dirs").map(|s| s.to_string()),
        copy_base: cmd_arguments.is_present("copy_base_memory"),
        copy_diff: cmd_arguments.is_present("copy_diff_memory"),
        kernel: cmd_arguments.value_of("kernel").expect("kernel").to_string(),
        cmdline: cmd_arguments.value_of("kernel_args").map(|s| s.to_string()),
    };
    let id: &str = cmd_arguments.value_of("id").expect("id");
    //println!("id: {}, function config: {:?}", id, vm_app_config);

    let odirect = snapfaas::vm::OdirectOption {
        base: cmd_arguments.is_present("odirect base"),
        diff: !cmd_arguments.is_present("no odirect diff"),
        rootfs: !cmd_arguments.is_present("no odirect rootfs"),
        appfs: !cmd_arguments.is_present("no odirect appfs")
    };
    // Launch a vm based on the FunctionConfig value
    let firerunner = cmd_arguments.value_of("firerunner").unwrap();
    use std::os::unix::io::FromRawFd;
    let (mut vm, _) = Vm::new(id, &vm_app_config, &unsafe{ UnixListener::from_raw_fd(-1) }, CID,
        cmd_arguments.value_of("network"), firerunner, cmd_arguments.is_present("force exit"), Some(odirect)).expect("Failed to create vm");

    vm.wait();
}
