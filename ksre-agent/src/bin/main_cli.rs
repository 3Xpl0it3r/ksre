use clap::{Args, Parser, Subcommand};
use ksre_lib::serializer::bytes::BytesCodec;
use libagent::collector::procfs::systeminfo::SystemInfo;
use libagent::storage::Store;

mod client;

#[derive(Parser)]
#[command(name = "ksre")]
struct Cli {
    #[arg(long)]
    start: Option<u64>,
    #[arg(long)]
    end: Option<u64>,
    #[command(flatten)]
    catalog: MetricCatalog,

    #[command(subcommand)]
    process: ProcessCommand,
}

#[derive(Subcommand)]
enum ProcessCommand {
    Process {},
}
// MetricCatalog[#TODO] (shoule add some comments )
#[derive(Args)]
#[group(multiple = false)]
struct MetricCatalog {
    #[arg(long)]
    cpu: bool,
    #[arg(long)]
    mem: bool,
    #[arg(long)]
    vm: bool,
    #[arg(long)]
    io: bool,
    #[arg(long)]
    tcp: bool,
    #[arg(long)]
    udp: bool,
}

fn main() {
    let store = Store::reader();
    let store_itera = store.range_query(9, 2);
    for item in store_itera {
        let mut system_info = SystemInfo::default();
        system_info.byte_decode(&item);
        println!("{:?}\n", system_info);
    }
}
