use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use crate::dns::config::DnsConfig;
use crate::dns::packet::{self, Message, PacketError};
use crate::dns::recursive::hints::RootHints;
use crate::dns::recursive::transport::UdpTransport;
use crate::dns::wire::parse_qname;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainValidationError {
    InvalidName,
    LookupFailed,
}

pub fn check_domain_registration(
    name: &str,
    dns_config: &DnsConfig,
) -> Result<(), DomainValidationError> {
    let name = name.trim().trim_end_matches('.');
    if name.is_empty() || name.split('.').count() < 2 {
        return Err(DomainValidationError::InvalidName);
    }

    resolve_ns_authoritative(name, dns_config)
        .map(|nameservers| {
            if nameservers.is_empty() {
                Err(DomainValidationError::LookupFailed)
            } else {
                Ok(())
            }
        })
        .unwrap_or(Err(DomainValidationError::LookupFailed))
}

/// Build a DNS query with RD=0.
fn build_auth_query(id: u16, qname: &str, qtype: u16) -> Result<Vec<u8>, PacketError> {
    let mut query = Message::query(id, qname, qtype);
    query.header.set_recursion_desired(false);
    query.encode()
}

/// Normalize a domain name for comparison: lowercase, strip trailing dot.
fn normalize_name(name: &str) -> String {
    name.trim_end_matches('.').to_ascii_lowercase()
}

fn record_name(resp: &[u8], record: &packet::Record) -> Option<String> {
    parse_qname(resp, record.rdata_offset).map(|(name, _)| name)
}

fn record_ip(record: &packet::Record) -> Option<String> {
    match record.rtype {
        packet::TYPE_A if record.rdata.len() == 4 => Some(
            Ipv4Addr::new(
                record.rdata[0],
                record.rdata[1],
                record.rdata[2],
                record.rdata[3],
            )
            .to_string(),
        ),
        packet::TYPE_AAAA if record.rdata.len() == 16 => {
            let mut octets = [0u8; 16];
            octets.copy_from_slice(&record.rdata);
            Some(Ipv6Addr::from(octets).to_string())
        }
        _ => None,
    }
}

/// Resolve A records for a nameserver name via authoritative-only resolution.
fn resolve_a_authoritative(ns_name: &str, dns_config: &DnsConfig) -> Result<Vec<String>, String> {
    let timeout = dns_config.udp_timeout.max(Duration::from_secs(2));
    let transport = UdpTransport::bind(timeout).ok_or("Failed to bind UDP socket")?;
    let hints = RootHints::load();

    let mut server_ips: Vec<String> = hints.servers().to_vec();

    for _ in 0..5 {
        let qid: u16 = rand::random();
        let Ok(query) = build_auth_query(qid, ns_name, packet::TYPE_A) else {
            continue;
        };

        for ip in &server_ips {
            let query_resp =
                transport.send_recv(ip, &query, qid, ns_name, packet::TYPE_A, dns_config);
            let Some(resp_bytes) = query_resp else {
                continue;
            };
            let Ok(msg) = Message::parse(&resp_bytes) else {
                continue;
            };

            // Check answer section for A records.
            let addrs: Vec<String> = msg.answers.iter().filter_map(record_ip).collect();
            if !addrs.is_empty() {
                return Ok(addrs);
            }

            // Follow referral, extract glue IPs.
            let ns_names: Vec<String> = msg
                .authorities
                .iter()
                .filter(|record| record.rtype == packet::TYPE_NS)
                .filter_map(|record| record_name(&resp_bytes, record))
                .collect();
            let glue_ips: Vec<String> = msg.additionals.iter().filter_map(record_ip).collect();

            if !ns_names.is_empty() {
                if !glue_ips.is_empty() {
                    server_ips = glue_ips;
                } else {
                    // No glue, try resolving NS names recursively.
                    let mut resolved = Vec::new();
                    for name in &ns_names {
                        if let Ok(ips) =
                            resolve_a_authoritative(name.trim_end_matches('.'), dns_config)
                        {
                            resolved.extend(ips);
                        }
                    }
                    if resolved.is_empty() {
                        return Err(format!("Cannot resolve NS: {}", ns_names.join(", ")));
                    }
                    server_ips = resolved;
                }
                break; // move to next iteration
            }
        }
    }

    Err("Could not resolve A record for nameserver".to_string())
}

pub fn resolve_ns_authoritative(
    qname: &str,
    dns_config: &DnsConfig,
) -> Result<Vec<String>, String> {
    let timeout = dns_config.udp_timeout.max(Duration::from_secs(2));
    let transport = UdpTransport::bind(timeout).ok_or("Failed to bind UDP socket")?;
    let hints = RootHints::load();

    let target = normalize_name(qname);
    let mut server_ips: Vec<String> = hints.servers().to_vec();

    for _ in 0..10 {
        let qid: u16 = rand::random();
        let Ok(query) = build_auth_query(qid, qname, packet::TYPE_NS) else {
            return Err("Invalid nameserver query name".to_string());
        };

        let mut got_response = false;
        let mut ns_names: Vec<String> = Vec::new();
        let mut glue_ips: Vec<String> = Vec::new();
        let mut delegation_name: Option<String> = None;

        for ip in &server_ips {
            let query_resp =
                transport.send_recv(ip, &query, qid, qname, packet::TYPE_NS, dns_config);
            let Some(resp_bytes) = query_resp else {
                continue;
            };
            let Ok(msg) = Message::parse(&resp_bytes) else {
                continue;
            };

            // Collect NS records from both authority and answer sections.
            let mut found: Vec<(String, String)> = Vec::new(); // (owner_name, ns_target)
            for record in msg.authorities.iter().chain(&msg.answers) {
                if record.rtype == packet::TYPE_NS
                    && let Some(ns) = record_name(&resp_bytes, record)
                {
                    found.push((record.name.clone(), ns));
                }
            }

            if found.is_empty() {
                continue;
            }

            got_response = true;

            // Check whether the delegation owner matches the queried domain.
            let owner = normalize_name(&found[0].0);
            if owner == target {
                return Ok(found.iter().map(|(_, ns)| ns.clone()).collect());
            }

            delegation_name = Some(owner);
            ns_names = found.iter().map(|(_, ns)| ns.clone()).collect();

            glue_ips.extend(msg.additionals.iter().filter_map(record_ip));
            break; // use first responding server
        }

        if !got_response {
            return Err("No NS delegation found from authoritative servers".to_string());
        }

        // Prepare next-hop server IPs.
        if !glue_ips.is_empty() {
            server_ips = glue_ips;
        } else {
            // No glue in referral, resolve NS names to IPs.
            let mut resolved = Vec::new();
            for name in &ns_names {
                if let Ok(ips) = resolve_a_authoritative(name.trim_end_matches('.'), dns_config) {
                    resolved.extend(ips);
                }
            }
            if resolved.is_empty() {
                return Err(format!(
                    "Cannot resolve nameservers for {}: {}",
                    delegation_name.unwrap_or_default(),
                    ns_names.join(", ")
                ));
            }
            server_ips = resolved;
        }
    }

    Err("Too many delegation levels walking authoritative chain".to_string())
}
