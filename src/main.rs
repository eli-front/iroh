use clap::{Parser, Subcommand, ValueEnum};
use tabled::{Style, Table, Tabled};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    //// Number of devices to show
    #[arg(short, long, default_value_t = 500)]
    limit: u16,

    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Elements {
    Interfaces,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List a set of requested elements
    #[command(arg_required_else_help = true, name = "ls")]
    List {
        /// Which element to list
        #[arg(name = "element", required = true)]
        element: Elements,
    },
}

#[derive(Tabled)]
struct Interface {
    name: String,
    ipv6: String,
    ipv4: String,
    is_up: bool,
    is_loopback: bool,
    is_multicast: bool,
    is_broadcast: bool,
    is_point_to_point: bool,
    is_running: bool,
}

impl Interface {
    fn new(interface: &pnet::datalink::NetworkInterface) -> Self {
        Self {
            name: interface.name.to_string(),
            ipv6: interface
                .ips
                .iter()
                .filter(|ip| ip.is_ipv6())
                .map(|ip| ip.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            ipv4: interface
                .ips
                .iter()
                .filter(|ip| ip.is_ipv4())
                .map(|ip| ip.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            is_up: interface.is_up(),
            is_loopback: interface.is_loopback(),
            is_multicast: interface.is_multicast(),
            is_broadcast: interface.is_broadcast(),
            is_point_to_point: interface.is_point_to_point(),
            is_running: interface.is_running(),
        }
    }
}

fn list_interfaces(args: Args) {
    let mut binding = pnet::datalink::interfaces();

    // limit number of interfaces from args
    binding.truncate(args.limit as usize);

    let interfaces = binding
        .iter_mut()
        .map(|interface| Interface::new(&interface));

    // print table header
    let mut table = Table::new(interfaces);

    table.with(Style::modern());

    println!("{}", table.to_string());
}

fn main() {
    let args = Args::parse();

    // switch on command
    match args.command {
        Commands::List { element } => match element {
            Elements::Interfaces => list_interfaces(args),
        },
    }
}
