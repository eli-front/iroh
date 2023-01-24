use std::collections::HashMap;

use clap::{Parser, Subcommand, ValueEnum};
use pnet::util::MacAddr;
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
    Networks,
    Wifi,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Scannable {
    Packets,
    Devices,
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

    /// Scan
    #[command(name = "scan", arg_required_else_help = true)]
    Scan {
        /// Which element to scan
        #[arg(name = "scannable", required = true)]
        scannable: Scannable,
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
    is_wifi: bool,
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
            is_wifi: is_wifi_interface(interface),
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

fn is_wifi_interface(interface: &pnet::datalink::NetworkInterface) -> bool {
    // FIXME: this is a hack to get the wifi interfaces
    interface.name.contains("en0")
}

fn get_wifi_interfaces() -> Vec<pnet::datalink::NetworkInterface> {
    let mut interfaces = pnet::datalink::interfaces();

    interfaces.retain(|interface| is_wifi_interface(interface));

    interfaces
}

fn list_wifi_interfaces() {
    let mut binding = get_wifi_interfaces();

    let interfaces = binding
        .iter_mut()
        .map(|interface| Interface::new(&interface));

    let mut table = Table::new(interfaces);
    table.with(Style::modern());

    println!("{}", table.to_string());
}

fn scan_packets() {
    let interface = get_wifi_interfaces()[0].clone();

    // scan for packets on network interface
    let (mut _tx, mut rx) = match pnet::datalink::channel(&interface, Default::default()) {
        Ok(pnet::datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Error: {}", e),
    };

    loop {
        match rx.next() {
            Ok(packet) => {
                let packet = pnet::packet::ethernet::EthernetPacket::new(packet).unwrap();

                println!("Got a packet: {:?}", packet);
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

fn get_device_info(mac: MacAddr) -> String {
    let mut mac = mac.to_string();
    // get first 3 bytes of mac address
    mac.truncate(8);

    // get file mac-vendors-export.csv and find row where mac prefix matches

    let bytes = include_bytes!("../static/mac-vendors-export.csv");

    let mut reader = csv::Reader::from_reader(bytes.as_ref());
    for result in reader.records() {
        let record = result.expect("Could not read record");

        // ignore case
        if record[0].to_ascii_uppercase() == mac.to_ascii_uppercase() {
            return record[1].to_string();
        }
    }

    return format!("Unknown device id: {}", mac);
}

async fn scan_devices() {
    let interface = get_wifi_interfaces()[0].clone();

    // scan for packets on network interface
    let (mut _tx, mut rx) = match pnet::datalink::channel(&interface, Default::default()) {
        Ok(pnet::datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Error: {}", e),
    };

    let mut devices = HashMap::new();

    loop {
        // clear screen

        match rx.next() {
            Ok(packet) => {
                let packet = pnet::packet::ethernet::EthernetPacket::new(packet).unwrap();

                let source = packet.get_source();
                let destination = packet.get_destination();

                // add source and destination to devices hashmap map to device info
                // if not already in hashmap
                if !devices.contains_key(&source) {
                    let info = get_device_info(source);

                    devices.insert(source, info.clone());
                    println!("{}: {}", source, info)
                }

                if !devices.contains_key(&destination) {
                    let info = get_device_info(destination);

                    devices.insert(destination, info.clone());
                    println!("{}: {}", destination, info)
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // switch on command
    match args.command {
        Commands::List { element } => match element {
            // switch on element
            Elements::Networks => list_interfaces(args),
            Elements::Wifi => list_wifi_interfaces(),
        },
        Commands::Scan { scannable } => match scannable {
            Scannable::Packets => scan_packets(),
            Scannable::Devices => {
                scan_devices().await;
            }
        },
    }
}
