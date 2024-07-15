use std::usize;

// Catalog[#TODO] (shoule add some comments )
struct Catalog {
    cpu: bool,
    mem: bool,
    vm: bool,
    io: bool,
    tcp: bool,
    udp: bool,
}

// Options[#TODO] (shoule add some comments )
struct Opts {
    start_ns: Option<u64>,
    end_ns: Option<u64>,
    limits: Option<usize>,
}

enum ProcessCommand {
    Process{

    }
}
