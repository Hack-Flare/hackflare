use std::time::Duration;

use hickory_server::proto::op::Message;
use hickory_server::proto::rr::{RData, RecordType};

use crate::dns::config::DnsConfig;
use crate::dns::recursive::hints::RootHints;
use crate::dns::recursive::transport::UdpTransport;
use crate::dns::wire::encode_name_labels_vec;

/// Build a DNS query with RD=0 (no recursion)
fn build_auth_query(id: u16, qname: &str, qtype: u16) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&id.to_be_bytes());
    out.extend_from_slice(&0x0000u16.to_be_bytes()); // flags: RD=0
    out.extend_from_slice(&1u16.to_be_bytes()); // qdcount = 1
    out.extend_from_slice(&0u16.to_be_bytes()); // ancount = 0
    out.extend_from_slice(&0u16.to_be_bytes()); // nscount = 0
    out.extend_from_slice(&0u16.to_be_bytes()); // arcount = 0
    out.extend_from_slice(&encode_name_labels_vec(qname));
    out.extend_from_slice(&qtype.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes()); // qclass = IN
    out
}

/// Perform authoritative-only validation by querying the domain's parent zone
/// (TLD) nameservers directly with non-recursive queries.
///
/// Returns the NS delegation records as reported by the registry/parent zone.
pub fn resolve_ns_authoritative(
    qname: &str,
    dns_config: &DnsConfig,
) -> Result<Vec<String>, String> {
    let timeout = dns_config.udp_timeout.max(Duration::from_secs(2));

    // Query root servers (non-recursive) to discover TLD nameserver IPs.
    let transport = UdpTransport::bind(timeout).ok_or("Failed to bind UDP socket")?;
    let hints = RootHints::load();
    let root_servers = hints.servers().to_vec();

    let qid1: u16 = rand::random();
    let query1 = build_auth_query(qid1, qname, 2); // qtype = NS (2)

    let mut tld_ips: Vec<String> = Vec::new();
    for root_ip in &root_servers {
        let query_resp = transport.send_recv(root_ip, &query1, qid1, qname, 2, dns_config);
        let resp_bytes = match query_resp {
            Some(b) => b,
            None => continue,
        };
        let Ok(msg) = Message::from_vec(&resp_bytes) else {
            continue;
        };

        for rec in &msg.additionals {
            if rec.record_type() == RecordType::A
                && let RData::A(ip) = &rec.data
            {
                tld_ips.push(ip.to_string());
            } else if rec.record_type() == RecordType::AAAA
                && let RData::AAAA(ip) = &rec.data
            {
                tld_ips.push(ip.to_string());
            }
        }
        if !tld_ips.is_empty() {
            break;
        }
    }

    if tld_ips.is_empty() {
        return Err("Could not determine authoritative nameservers for TLD".to_string());
    }

    // Step 2: Query TLD nameserver (non-recursive) for the domain's NS delegation.
    let transport2 = UdpTransport::bind(timeout).ok_or("Failed to bind UDP socket")?;
    let qid2: u16 = rand::random();
    let query2 = build_auth_query(qid2, qname, 2);

    for tld_ip in &tld_ips {
        let query_resp = transport2.send_recv(tld_ip, &query2, qid2, qname, 2, dns_config);
        let resp_bytes = match query_resp {
            Some(b) => b,
            None => continue,
        };
        let Ok(msg) = Message::from_vec(&resp_bytes) else {
            continue;
        };

        let mut ns_names: Vec<String> = Vec::new();

        // Check authority section for delegation NS records
        for rec in &msg.authorities {
            if rec.record_type() == RecordType::NS
                && let RData::NS(ns) = &rec.data
            {
                ns_names.push(ns.to_utf8());
            }
        }

        // Also check answer section (some servers put NS delegation in answers)
        for rec in &msg.answers {
            if rec.record_type() == RecordType::NS
                && let RData::NS(ns) = &rec.data
            {
                ns_names.push(ns.to_utf8());
            }
        }

        if !ns_names.is_empty() {
            return Ok(ns_names);
        }
    }

    Err("No NS delegation found from authoritative TLD servers".to_string())
}
