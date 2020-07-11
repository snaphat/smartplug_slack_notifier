
mod notifier;
mod service;

static IPS: &'static [&str] = &[
    //"192.168.2.147:9999", // Clab-room-light
    "192.168.2.146:9999", // SC6800-NSLEDS1
    "192.168.2.101:9999", // SC6800-NSLEDS3
];

fn add_hosts() -> notifier::Hosts {
    let mut hosts = notifier::Hosts::new();

    // Add IPs to notifier.
    for ip in IPS.iter() {
        hosts.add(ip);
    }
    hosts
}

#[cfg(windows)]
fn main() {
    let mut hosts = add_hosts();
    match service::run(move || hosts.check()) {
        Ok(_) => (),
        Err(_) => ()
    }
}

#[cfg(not(windows))]
fn main() {
    let mut hosts = add_hosts();
    loop {
        check_hosts(&mut hosts);
    }
}
