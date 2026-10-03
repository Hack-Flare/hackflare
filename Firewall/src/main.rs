mod config;
mod dns_inspector;
mod firewall;
mod ids;
mod ips;

use config::load_config;
use dns_inspector::{DnsDecision, DnsInspector};
use firewall::{Action, FirewallEngine};
use ids::IdsEngine;
use ips::IpsEngine;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Failed to load config: {err}");
            process::exit(1);
        }
    };

    let firewall = FirewallEngine::new(
        config.firewall.name.clone(),
        config.firewall.default_policy.input.clone(),
        config.firewall.default_policy.forward.clone(),
        config.firewall.default_policy.output.clone(),
        config.firewall.allowlist.clone(),
        config.firewall.blacklist.clone(),
        config.firewall.blocked_ports.clone(),
        config.firewall.logging.enabled,
        config.firewall.nat.enabled,
        config.firewall.rate_limit.enabled,
        config.firewall.rate_limit.requests_per_second,
    );

    let dns_inspector = DnsInspector::new(
        config.dns.blocked_domains.clone(),
        config.dns.allowed_tlds.clone(),
        config.dns.sinkhole_ip.clone(),
        config.dns.enabled,
        config.dns.log_queries,
    );

    let mut ids = IdsEngine::new();
    let mut ips = IpsEngine::new(firewall.clone());

    match command.as_str() {
        "start" => {
            println!("Starting Hackflare firewall: {}", ips.firewall().name());
            println!("{}", ips.firewall().generate_rules());
            println!("DNS inspection enabled: {}", config.dns.enabled);
            println!("IDS + IPS connected and active.");
        }
        "show" => {
            println!("{}", ips.firewall().generate_rules());
        }
        "generate" => {
            println!("{}", ips.firewall().generate_rules());
        }
        "dns-check" => {
            let target = args.get(2).map(String::as_str).unwrap_or("example.com");
            match dns_inspector.inspect(target) {
                DnsDecision::Allow => println!("DNS query for {} is allowed.", target),
                DnsDecision::Block => {
                    println!("DNS query for {} is blocked and redirected to {}.", target, dns_inspector.sinkhole_ip());
                    let _ = ips.handle_dns_decision("client", target, DnsDecision::Block);
                }
                DnsDecision::LogOnly => {
                    println!("DNS query for {} is suspicious and will be logged.", target);
                    let _ = ips.handle_dns_decision("client", target, DnsDecision::LogOnly);
                }
            }
        }
        "test" => {
            let action = ips.firewall().evaluate("8.8.8.8", "192.168.1.25", "tcp", 22);
            match action {
                Action::Allow => println!("Test packet allowed."),
                Action::Drop => println!("Test packet dropped."),
                Action::LogDrop => println!("Test packet dropped and logged."),
            }

            if let Some(event) = ids.record_syn("203.0.113.8") {
                let _ = ips.analyze_event(event);
            }

            let domain = "malware.example";
            match dns_inspector.inspect(domain) {
                DnsDecision::Allow => println!("Domain is allowed."),
                DnsDecision::Block => println!("Domain is blocked."),
                DnsDecision::LogOnly => println!("Domain is suspicious."),
            }
        }
        "stop" => {
            println!("Stopping Hackflare firewall: {}", ips.firewall().name());
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Usage: hackflare <start|show|generate|dns-check|test|stop> [domain]");
}
