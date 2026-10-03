use crate::dns_inspector::DnsDecision;
use crate::firewall::FirewallEngine;
use crate::ids::{IdsEngine, NetworkEvent, ThreatType};

#[derive(Debug, Clone)]
pub struct IpsEngine {
    firewall: FirewallEngine,
    ids: IdsEngine,
}

impl IpsEngine {
    pub fn new(firewall: FirewallEngine) -> Self {
        Self {
            firewall,
            ids: IdsEngine::new(),
        }
    }

    pub fn handle_packet(
        &mut self,
        source_ip: &str,
        destination_ip: &str,
        protocol: &str,
        destination_port: u16,
        is_syn: bool,
        is_dns: bool,
        dns_domain: Option<&str>,
        dns_decision: Option<DnsDecision>,
    ) -> bool {
        if let Some(event) = self.ids.process_packet(
            source_ip,
            destination_ip,
            protocol,
            destination_port,
            is_syn,
            is_dns,
            dns_domain,
            dns_decision,
        ) {
            return self.analyze_event(event);
        }

        false
    }

    pub fn analyze_event(&mut self, event: NetworkEvent) -> bool {
        match event.threat {
            ThreatType::SynFlood | ThreatType::PortScan => {
                let _ = self.firewall.add_blacklist(event.source_ip.clone());
                println!("[IPS] Blocking {} for malicious activity.", event.source_ip);
                return true;
            }
            ThreatType::SuspiciousDns => {
                println!("[IPS] DNS anomaly detected for {}.", event.source_ip);
                return true;
            }
        }
    }

    pub fn handle_dns_decision(&mut self, source_ip: &str, domain: &str, decision: DnsDecision) -> bool {
        if matches!(decision, DnsDecision::Block | DnsDecision::LogOnly) {
            println!("[IPS] DNS policy triggered for {} -> {}.", source_ip, domain);
            return true;
        }

        false
    }

    pub fn ids(&self) -> &IdsEngine {
        &self.ids
    }

    pub fn firewall(&self) -> &FirewallEngine {
        &self.firewall
    }
}
