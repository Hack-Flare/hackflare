use std::time::Duration;

use hickory_server::proto::op::Message;
use hickory_server::proto::rr::{RData, RecordType};

use crate::dns::config::DnsConfig;
use crate::dns::recursive::hints::RootHints;
use crate::dns::recursive::transport::UdpTransport;
use crate::dns::wire::encode_name_labels_vec;

/// Build a DNS query with RD=0
fn build_auth_query(id: u16, qname: &str, qtype: u16) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&id.to_be_bytes());
    out.extend_from_slice(&0x0000u16.to_be_bytes()); // flags: RD=0
    out.extend_from_slice(&1u16.to_be_bytes());  // qdcount = 1
    out.extend_from_slice(&0u16.to_be_bytes());  // ancount = 0
    out.extend_from_slice(&0u16.to_be_bytes());  // nscount = 0
    out.extend_from_slice(&0u16.to_be_bytes());  // arcount = 0
    out.extend_from_slice(&encode_name_labels_vec(qname));
    out.extend_from_slice(&qtype.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());  // qclass = IN
    out
}

/// Normalize a domain name for comparison: lowercase, strip trailing dot.
fn normalize_name(name: &str) -> String {
    name.trim_end_matches('.').to_ascii_lowercase()
}

/// Resolve A records for a nameserver name via authoritative-only resolution
fn resolve_a_authoritative(
    ns_name: &str,
    dns_config: &DnsConfig,
) -> Result<Vec<String>, String> {
    let timeout = dns_config.udp_timeout.max(Duration::from_secs(2));
    let transport = UdpTransport::bind(timeout).ok_or("Failed to bind UDP socket")?;
    let hints = RootHints::load();

    let mut server_ips: Vec<String> = hints.servers().to_vec();

    for _ in 0..5 {
        let qid: u16 = rand::random();
        let query = build_auth_query(qid, ns_name, 1); // qtype = A (1)

        for ip in &server_ips {
            let query_resp = transport.send_recv(ip, &query, qid, ns_name, 1, dns_config);
            let resp_bytes = match query_resp {
                Some(b) => b,
                None => continue,
            };
            let Ok(msg) = Message::from_vec(&resp_bytes) else { continue };

            // Check answer section for A records
            let mut addrs: Vec<String> = Vec::new();
            for rec in &msg.answers {
                if rec.record_type() == RecordType::A && let RData::A(ip) = &rec.data {
                    addrs.push(ip.to_string());
                }
            }
            if !addrs.is_empty() {
                return Ok(addrs);
            }

            // Follow referral, extract glue IPs
            let mut ns_names: Vec<String> = Vec::new();
            let mut glue_ips: Vec<String> = Vec::new();
            for rec in &msg.authorities {
                if rec.record_type() == RecordType::NS && let RData::NS(ns) = &rec.data {
                    ns_names.push(ns.to_utf8());
                }
            }
            for rec in &msg.additionals {
                if rec.record_type() == RecordType::A && let RData::A(ip) = &rec.data {
                    glue_ips.push(ip.to_string());
                } else if rec.record_type() == RecordType::AAAA && let RData::AAAA(ip) = &rec.data {
                    glue_ips.push(ip.to_string());
                }
            }

            if !ns_names.is_empty() {
                if !glue_ips.is_empty() {
                    server_ips = glue_ips;
                } else {
                    // No glue, try resolving NS names recursively
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
        let query = build_auth_query(qid, qname, 2); // qtype = NS (2)

        let mut got_response = false;
        let mut ns_names: Vec<String> = Vec::new();
        let mut glue_ips: Vec<String> = Vec::new();
        let mut delegation_name: Option<String> = None;

        for ip in &server_ips {
            let query_resp = transport.send_recv(ip, &query, qid, qname, 2, dns_config);
            let resp_bytes = match query_resp {
                Some(b) => b,
                None => continue,
            };
            let Ok(msg) = Message::from_vec(&resp_bytes) else { continue };

            // Collect NS records from both authority and answer sections.
            let mut found: Vec<(String, String)> = Vec::new(); // (owner_name, ns_target)
            for rec in &msg.authorities {
                if rec.record_type() == RecordType::NS
                    && let RData::NS(ns) = &rec.data
                {
                    found.push((rec.name.to_utf8(), ns.to_utf8()));
                }
            }
            for rec in &msg.answers {
                if rec.record_type() == RecordType::NS
                    && let RData::NS(ns) = &rec.data
                {
                    found.push((rec.name.to_utf8(), ns.to_utf8()));
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

            for rec in &msg.additionals {
                if rec.record_type() == RecordType::A
                    && let RData::A(ip) = &rec.data
                {
                    glue_ips.push(ip.to_string());
                } else if rec.record_type() == RecordType::AAAA
                    && let RData::AAAA(ip) = &rec.data
                {
                    glue_ips.push(ip.to_string());
                }
            }
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
