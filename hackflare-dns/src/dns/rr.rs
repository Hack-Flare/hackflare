//! DNS record types and resource data.
//!
//! These types back the in-memory zone store and are independent of the wire codec.
//! Names are stored as canonical lowercase fully qualified strings with a trailing dot.

use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use crate::dns::packet;
use crate::dns::wire::encode_name_labels_vec;

/// DNS resource record types supported by the zone store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RecordType {
    A,
    Ns,
    Cname,
    Soa,
    Ptr,
    Mx,
    Txt,
    Aaaa,
    Srv,
    Aname,
    Opt,
    Caa,
    Ixfr,
    Axfr,
    Any,
    Unknown(u16),
}

impl RecordType {
    #[must_use]
    pub const fn from_u16(value: u16) -> Self {
        match value {
            1 => Self::A,
            2 => Self::Ns,
            5 => Self::Cname,
            6 => Self::Soa,
            12 => Self::Ptr,
            15 => Self::Mx,
            16 => Self::Txt,
            28 => Self::Aaaa,
            33 => Self::Srv,
            41 => Self::Opt,
            251 => Self::Ixfr,
            252 => Self::Axfr,
            255 => Self::Any,
            257 => Self::Caa,
            65305 => Self::Aname,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn as_u16(self) -> u16 {
        match self {
            Self::A => 1,
            Self::Ns => 2,
            Self::Cname => 5,
            Self::Soa => 6,
            Self::Ptr => 12,
            Self::Mx => 15,
            Self::Txt => 16,
            Self::Aaaa => 28,
            Self::Srv => 33,
            Self::Opt => 41,
            Self::Ixfr => 251,
            Self::Axfr => 252,
            Self::Any => 255,
            Self::Caa => 257,
            Self::Aname => 65305,
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for RecordType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let upper = value.trim().to_ascii_uppercase();
        match upper.as_str() {
            "A" => Ok(Self::A),
            "NS" => Ok(Self::Ns),
            "CNAME" => Ok(Self::Cname),
            "SOA" => Ok(Self::Soa),
            "PTR" => Ok(Self::Ptr),
            "MX" => Ok(Self::Mx),
            "TXT" => Ok(Self::Txt),
            "AAAA" => Ok(Self::Aaaa),
            "SRV" => Ok(Self::Srv),
            "ANAME" => Ok(Self::Aname),
            "OPT" => Ok(Self::Opt),
            "CAA" => Ok(Self::Caa),
            "IXFR" => Ok(Self::Ixfr),
            "AXFR" => Ok(Self::Axfr),
            "ANY" => Ok(Self::Any),
            other => other
                .strip_prefix("TYPE")
                .and_then(|number| number.parse::<u16>().ok())
                .map_or(Err(()), |number| Ok(Self::Unknown(number))),
        }
    }
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A => f.write_str("A"),
            Self::Ns => f.write_str("NS"),
            Self::Cname => f.write_str("CNAME"),
            Self::Soa => f.write_str("SOA"),
            Self::Ptr => f.write_str("PTR"),
            Self::Mx => f.write_str("MX"),
            Self::Txt => f.write_str("TXT"),
            Self::Aaaa => f.write_str("AAAA"),
            Self::Srv => f.write_str("SRV"),
            Self::Aname => f.write_str("ANAME"),
            Self::Opt => f.write_str("OPT"),
            Self::Caa => f.write_str("CAA"),
            Self::Ixfr => f.write_str("IXFR"),
            Self::Axfr => f.write_str("AXFR"),
            Self::Any => f.write_str("ANY"),
            Self::Unknown(value) => write!(f, "TYPE{value}"),
        }
    }
}

/// Canonicalize a domain name to lowercase with a trailing dot.
///
/// Returns `None` when a label is empty or too long, or when the name exceeds 255 bytes
#[must_use]
pub fn canonical_name(name: &str) -> Option<String> {
    let trimmed = name.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        return Some(".".to_string());
    }

    let mut wire_len = 1usize;
    for label in trimmed.split('.') {
        if label.is_empty() || label.len() > 63 {
            return None;
        }
        wire_len += 1 + label.len();
    }
    if wire_len > 255 {
        return None;
    }

    Some(format!("{}.", trimmed.to_ascii_lowercase()))
}

/// Resource record data supported by the zone store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RData {
    A(Ipv4Addr),
    Aaaa(Ipv6Addr),
    Ns(String),
    Cname(String),
    Ptr(String),
    Aname(String),
    Mx {
        preference: u16,
        exchange: String,
    },
    Txt(Vec<String>),
    Soa {
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    },
    Srv {
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    },
    Caa {
        issuer_critical: bool,
        reserved_flags: u8,
        tag: String,
        value: Vec<u8>,
    },
}

impl RData {
    #[must_use]
    pub const fn rtype(&self) -> RecordType {
        match self {
            Self::A(_) => RecordType::A,
            Self::Aaaa(_) => RecordType::Aaaa,
            Self::Ns(_) => RecordType::Ns,
            Self::Cname(_) => RecordType::Cname,
            Self::Ptr(_) => RecordType::Ptr,
            Self::Aname(_) => RecordType::Aname,
            Self::Mx { .. } => RecordType::Mx,
            Self::Txt(_) => RecordType::Txt,
            Self::Soa { .. } => RecordType::Soa,
            Self::Srv { .. } => RecordType::Srv,
            Self::Caa { .. } => RecordType::Caa,
        }
    }

    /// Parse record data from its presentation format.
    #[must_use]
    pub fn parse(rtype: RecordType, data: &str) -> Option<Self> {
        match rtype {
            RecordType::A => data.trim().parse::<Ipv4Addr>().ok().map(Self::A),
            RecordType::Aaaa => data.trim().parse::<Ipv6Addr>().ok().map(Self::Aaaa),
            RecordType::Ns => canonical_name(data).map(Self::Ns),
            RecordType::Cname => canonical_name(data).map(Self::Cname),
            RecordType::Ptr => canonical_name(data).map(Self::Ptr),
            RecordType::Aname => canonical_name(data).map(Self::Aname),
            RecordType::Mx => {
                let mut parts = data.split_whitespace();
                let preference = parts.next()?.parse::<u16>().ok()?;
                let exchange = canonical_name(parts.next()?)?;
                if parts.next().is_some() {
                    return None;
                }
                Some(Self::Mx {
                    preference,
                    exchange,
                })
            }
            RecordType::Txt => Some(Self::Txt(parse_txt(data))),
            RecordType::Soa => {
                let mut parts = data.split_whitespace();
                let mname = canonical_name(parts.next()?)?;
                let rname = canonical_name(parts.next()?)?;
                let serial = parts.next()?.parse::<u32>().ok()?;
                let refresh = parts.next()?.parse::<u32>().ok()?;
                let retry = parts.next()?.parse::<u32>().ok()?;
                let expire = parts.next()?.parse::<u32>().ok()?;
                let minimum = parts.next()?.parse::<u32>().ok()?;
                if parts.next().is_some() {
                    return None;
                }
                Some(Self::Soa {
                    mname,
                    rname,
                    serial,
                    refresh,
                    retry,
                    expire,
                    minimum,
                })
            }
            RecordType::Srv => {
                let mut parts = data.split_whitespace();
                let priority = parts.next()?.parse::<u16>().ok()?;
                let weight = parts.next()?.parse::<u16>().ok()?;
                let port = parts.next()?.parse::<u16>().ok()?;
                let target = canonical_name(parts.next()?)?;
                if parts.next().is_some() {
                    return None;
                }
                Some(Self::Srv {
                    priority,
                    weight,
                    port,
                    target,
                })
            }
            RecordType::Caa => {
                let mut parts = data.splitn(3, char::is_whitespace);
                let flags = parts.next()?.parse::<u8>().ok()?;
                let tag = parts.next()?.trim();
                if tag.is_empty() || tag.len() > 255 {
                    return None;
                }
                let value = parts.next().map_or("", str::trim);
                let value = strip_quotes(value);
                Some(Self::Caa {
                    issuer_critical: flags & 0x80 != 0,
                    reserved_flags: flags & 0x7F,
                    tag: tag.to_string(),
                    value: value.as_bytes().to_vec(),
                })
            }
            RecordType::Opt
            | RecordType::Ixfr
            | RecordType::Axfr
            | RecordType::Any
            | RecordType::Unknown(_) => None,
        }
    }

    /// Encode RDATA to DNS wire format. Names are uncompressed.
    #[must_use]
    pub fn to_wire(&self) -> Vec<u8> {
        match self {
            Self::A(address) => address.octets().to_vec(),
            Self::Aaaa(address) => address.octets().to_vec(),
            Self::Ns(name) | Self::Cname(name) | Self::Ptr(name) | Self::Aname(name) => {
                encode_name_labels_vec(name)
            }
            Self::Mx {
                preference,
                exchange,
            } => {
                let mut out = preference.to_be_bytes().to_vec();
                out.extend_from_slice(&encode_name_labels_vec(exchange));
                out
            }
            Self::Txt(values) => {
                let mut out = Vec::new();
                for value in values {
                    let bytes = value.as_bytes();
                    if bytes.is_empty() {
                        out.push(0);
                        continue;
                    }
                    for chunk in bytes.chunks(255) {
                        out.push(chunk.len() as u8);
                        out.extend_from_slice(chunk);
                    }
                }
                out
            }
            Self::Soa {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum,
            } => {
                let mut out = encode_name_labels_vec(mname);
                out.extend_from_slice(&encode_name_labels_vec(rname));
                out.extend_from_slice(&serial.to_be_bytes());
                out.extend_from_slice(&refresh.to_be_bytes());
                out.extend_from_slice(&retry.to_be_bytes());
                out.extend_from_slice(&expire.to_be_bytes());
                out.extend_from_slice(&minimum.to_be_bytes());
                out
            }
            Self::Srv {
                priority,
                weight,
                port,
                target,
            } => {
                let mut out = priority.to_be_bytes().to_vec();
                out.extend_from_slice(&weight.to_be_bytes());
                out.extend_from_slice(&port.to_be_bytes());
                out.extend_from_slice(&encode_name_labels_vec(target));
                out
            }
            Self::Caa {
                issuer_critical,
                reserved_flags,
                tag,
                value,
            } => {
                let flags = (u8::from(*issuer_critical) << 7) | reserved_flags;
                let tag_bytes = tag.as_bytes();
                let tag_len = tag_bytes.len().min(usize::from(u8::MAX));
                let mut out = Vec::with_capacity(2 + tag_len + value.len());
                out.push(flags);
                out.push(tag_len as u8);
                out.extend_from_slice(&tag_bytes[..tag_len]);
                out.extend_from_slice(value);
                out
            }
        }
    }
}

impl fmt::Display for RData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A(address) => write!(f, "{address}"),
            Self::Aaaa(address) => write!(f, "{address}"),
            Self::Ns(name) | Self::Cname(name) | Self::Ptr(name) | Self::Aname(name) => {
                f.write_str(name)
            }
            Self::Mx {
                preference,
                exchange,
            } => write!(f, "{preference} {exchange}"),
            Self::Txt(values) => write!(f, "{}", values.join(" ")),
            Self::Soa {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum,
            } => write!(
                f,
                "{mname} {rname} {serial} {refresh} {retry} {expire} {minimum}"
            ),
            Self::Srv {
                priority,
                weight,
                port,
                target,
            } => write!(f, "{priority} {weight} {port} {target}"),
            Self::Caa {
                issuer_critical,
                reserved_flags,
                tag,
                value,
            } => {
                let flags = (u8::from(*issuer_critical) << 7) | reserved_flags;
                write!(f, "{flags} {tag} {}", String::from_utf8_lossy(value))
            }
        }
    }
}

/// A resource record stored in a zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub ttl: u32,
    pub data: RData,
}

impl Record {
    #[must_use]
    pub const fn record_type(&self) -> RecordType {
        self.data.rtype()
    }

    /// Convert the record into the wire record representation used by responses.
    #[must_use]
    pub fn to_wire(&self) -> packet::Record {
        packet::Record {
            name: self.name.clone(),
            rtype: self.record_type().as_u16(),
            class: 1,
            ttl: self.ttl,
            rdata: self.data.to_wire(),
            rdata_offset: 0,
        }
    }
}

fn parse_txt(data: &str) -> Vec<String> {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return vec![String::new()];
    }
    if !trimmed.contains('"') {
        return vec![trimmed.to_string()];
    }

    let mut values = Vec::new();
    let mut chars = trimmed.chars().peekable();
    while let Some(&next) = chars.peek() {
        if next.is_whitespace() {
            chars.next();
            continue;
        }
        if next == '"' {
            chars.next();
            let mut value = String::new();
            while let Some(character) = chars.next() {
                match character {
                    '\\' => {
                        if let Some(escaped) = chars.next() {
                            value.push(escaped);
                        }
                    }
                    '"' => break,
                    other => value.push(other),
                }
            }
            values.push(value);
        } else {
            let mut value = String::new();
            while let Some(&character) = chars.peek() {
                if character.is_whitespace() {
                    break;
                }
                value.push(character);
                chars.next();
            }
            values.push(value);
        }
    }

    if values.is_empty() {
        values.push(String::new());
    }
    values
}

fn strip_quotes(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}
