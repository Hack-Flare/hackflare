use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Allow,
    Drop,
    LogDrop,
}

#[derive(Debug, Clone)]
pub struct FirewallEngine {
    name: String,
    default_input: String,
    default_forward: String,
    default_output: String,
    allowlist: HashSet<String>,
    blacklist: HashSet<String>,
    blocked_ports: HashSet<u16>,
    logging_enabled: bool,
    nat_enabled: bool,
    rate_limit_enabled: bool,
    rate_limit_rps: u32,
    last_seen_by_ip: HashMap<String, Instant>,
}

#[derive(Debug, Clone)]
pub struct PacketSummary {
    pub src_ip: String,
    pub dst_ip: String,
    pub protocol: String,
    pub dst_port: u16,
}

impl FirewallEngine {
    pub fn new(
        name: String,
        default_input: String,
        default_forward: String,
        default_output: String,
        allowlist: Vec<String>,
        blacklist: Vec<String>,
        blocked_ports: Vec<u16>,
        logging_enabled: bool,
        nat_enabled: bool,
        rate_limit_enabled: bool,
        rate_limit_rps: u32,
    ) -> Self {
        Self {
            name,
            default_input,
            default_forward,
            default_output,
            allowlist: allowlist.into_iter().collect(),
            blacklist: blacklist.into_iter().collect(),
            blocked_ports: blocked_ports.into_iter().collect(),
            logging_enabled,
            nat_enabled,
            rate_limit_enabled,
            rate_limit_rps,
            last_seen_by_ip: HashMap::new(),
        }
    }

    pub fn generate_rules(&self) -> String {
        let mut rules = Vec::new();
        rules.push(format!("table inet {} {{", self.name));
        rules.push("  chain input {".to_string());
        rules.push(format!(
            "    type filter hook input priority 0; policy {};",
            self.default_input
        ));
        rules.push("    iifname \"lo\" accept".to_string());
        rules.push("    ct state invalid drop".to_string());
        rules.push("    ct state { established, related } accept".to_string());

        for ip in &self.allowlist {
            rules.push(format!("    ip saddr {} accept", ip));
        }

        for ip in &self.blacklist {
            rules.push(format!("    ip saddr {} drop", ip));
        }

        for port in &self.blocked_ports {
            rules.push(format!("    tcp dport {} drop", port));
        }

        if self.logging_enabled {
            rules.push("    log prefix \"HACKFLARE DROP: \" level warning".to_string());
            rules.push("    drop".to_string());
        }

        rules.push("  }".to_string());
        rules.push("  chain forward {".to_string());
        rules.push(format!(
            "    type filter hook forward priority 0; policy {};",
            self.default_forward
        ));
        rules.push("    ct state invalid drop".to_string());
        rules.push("    ct state { established, related } accept".to_string());
        rules.push("  }".to_string());
        rules.push("  chain output {".to_string());
        rules.push(format!(
            "    type filter hook output priority 0; policy {};",
            self.default_output
        ));
        rules.push("  }".to_string());

        if self.nat_enabled {
            rules.push("  chain postrouting { type nat hook postrouting priority 100; policy accept; masquerade; }".to_string());
        }

        rules.push("}".to_string());
        rules.join("\n")
    }

    pub fn evaluate(&self, src_ip: &str, dst_ip: &str, proto: &str, dst_port: u16) -> Action {
        if self.blacklist.contains(src_ip) || self.blacklist.contains(dst_ip) {
            return Action::Drop;
        }

        if self.blocked_ports.contains(&dst_port) {
            return Action::Drop;
        }

        if self.allowlist.iter().any(|entry| entry == src_ip || entry == dst_ip) {
            return Action::Allow;
        }

        if self.rate_limit_enabled && matches!(proto, "tcp" | "udp") {
            let now = Instant::now();
            let key = format!("{src_ip}:{proto}:{dst_ip}:{dst_port}");
            let rate_limit_window = 1.0 / (self.rate_limit_rps.max(1) as f64);
            if self
                .last_seen_by_ip
                .get(&key)
                .map(|last| now.duration_since(*last).as_secs_f64() < rate_limit_window)
                .unwrap_or(false)
            {
                return Action::LogDrop;
            }
        }

        Action::Allow
    }

    pub fn add_blacklist(&mut self, ip: String) -> bool {
        if self.allowlist.contains(&ip) {
            return false;
        }
        self.blacklist.insert(ip)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn summarize_packet(&self, src_ip: &str, dst_ip: &str, protocol: &str, dst_port: u16) -> PacketSummary {
        PacketSummary {
            src_ip: src_ip.to_string(),
            dst_ip: dst_ip.to_string(),
            protocol: protocol.to_string(),
            dst_port,
        }
    }

    pub fn should_block_packet(&self, src_ip: &str, dst_ip: &str, protocol: &str, dst_port: u16) -> bool {
        matches!(self.evaluate(src_ip, dst_ip, protocol, dst_port), Action::Drop | Action::LogDrop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_blacklisted_source() {
        let engine = FirewallEngine::new(
            "hackflare-fw".to_string(),
            "drop".to_string(),
            "drop".to_string(),
            "accept".to_string(),
            vec!["10.0.0.0/8".to_string()],
            vec!["203.0.113.10".to_string()],
            vec![22],
            true,
            true,
            true,
            100,
        );

        assert_eq!(engine.evaluate("203.0.113.10", "10.0.0.5", "tcp", 80), Action::Drop);
    }

    #[test]
    fn drops_blocked_port() {
        let engine = FirewallEngine::new(
            "hackflare-fw".to_string(),
            "drop".to_string(),
            "drop".to_string(),
            "accept".to_string(),
            vec![],
            vec![],
            vec![22],
            true,
            true,
            false,
            0,
        );

        assert_eq!(engine.evaluate("8.8.8.8", "1.1.1.1", "tcp", 22), Action::Drop);
    }
}
