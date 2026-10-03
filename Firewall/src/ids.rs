use crate::dns_inspector::DnsDecision;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreatType {
    SynFlood,
    PortScan,
    SuspiciousDns,
}

#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub source_ip: String,
    pub destination_ip: String,
    pub protocol: String,
    pub destination_port: u16,
    pub threat: ThreatType,
}

#[derive(Debug, Clone, Default)]
pub struct IdsEngine {
    syn_counts: HashMap<String, usize>,
    port_scan_counts: HashMap<String, usize>,
}

impl IdsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_packet(
        &mut self,
        source_ip: &str,
        destination_ip: &str,
        protocol: &str,
        destination_port: u16,
        is_syn: bool,
        is_dns: bool,
        dns_domain: Option<&str>,
        dns_decision: Option<DnsDecision>,
    ) -> Option<NetworkEvent> {
        if is_syn {
            let count = self.syn_counts.entry(source_ip.to_string()).or_insert(0);
            *count += 1;

            if *count >= 20 {
                self.syn_counts.insert(source_ip.to_string(), 0);
                return Some(NetworkEvent {
                    source_ip: source_ip.to_string(),
                    destination_ip: destination_ip.to_string(),
                    protocol: protocol.to_string(),
                    destination_port,
                    threat: ThreatType::SynFlood,
                });
            }
        }

        if destination_port == 0 && protocol == "tcp" {
            let count = self.port_scan_counts.entry(source_ip.to_string()).or_insert(0);
            *count += 1;

            if *count >= 10 {
                self.port_scan_counts.insert(source_ip.to_string(), 0);
                return Some(NetworkEvent {
                    source_ip: source_ip.to_string(),
                    destination_ip: destination_ip.to_string(),
                    protocol: protocol.to_string(),
                    destination_port,
                    threat: ThreatType::PortScan,
                });
            }
        }

        if is_dns {
            if let Some(decision) = dns_decision {
                if matches!(decision, DnsDecision::LogOnly | DnsDecision::Block) {
                    return Some(NetworkEvent {
                        source_ip: source_ip.to_string(),
                        destination_ip: destination_ip.to_string(),
                        protocol: "dns".to_string(),
                        destination_port: 53,
                        threat: ThreatType::SuspiciousDns,
                    });
                }
            }

            if let Some(domain) = dns_domain {
                let suspicious = domain.contains("malware")
                    || domain.contains("phishing")
                    || domain.contains("r57")
                    || domain.contains("shell");
                if suspicious {
                    return Some(NetworkEvent {
                        source_ip: source_ip.to_string(),
                        destination_ip: destination_ip.to_string(),
                        protocol: "dns".to_string(),
                        destination_port: 53,
                        threat: ThreatType::SuspiciousDns,
                    });
                }
            }
        }

        None
    }
}
