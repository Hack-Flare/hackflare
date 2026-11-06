//! DNS wire-format message parser and encoder.
//!
//! This module owns the DNS packet format used by the server: the header, question section,
//! resource records, and EDNS(0) OPT pseudo-records. Name compression is supported for owner
//! names and question names. Records parsed from a message keep their RDATA as raw bytes, so
//! callers that need a specific record type can decode it using the source message buffer.

use std::collections::HashMap;
use std::fmt;

use crate::dns::wire::parse_qname;

/// Fixed DNS header length in bytes.
pub const HEADER_LEN: usize = 12;
/// Largest DNS message that can be framed on TCP.
pub const MAX_MESSAGE_LEN: usize = u16::MAX as usize;
/// Classic DNS UDP payload size when EDNS is not used.
pub const CLASSIC_UDP_PAYLOAD: usize = 512;
/// Largest name that may appear on the wire, including the root label.
pub const MAX_NAME_LEN: usize = 255;
/// Maximum number of labels in a DNS name.
pub const MAX_LABELS: usize = 127;
/// Record type codes used by the server path.
pub const TYPE_A: u16 = 1;
pub const TYPE_NS: u16 = 2;
pub const TYPE_SOA: u16 = 6;
pub const TYPE_AAAA: u16 = 28;
pub const TYPE_OPT: u16 = 41;
pub const TYPE_IXFR: u16 = 251;
pub const TYPE_AXFR: u16 = 252;
pub const TYPE_ANY: u16 = 255;

/// DNS header flag bits.
pub mod flags {
    pub const QR: u16 = 0x8000;
    pub const OPCODE_MASK: u16 = 0x7800;
    pub const AA: u16 = 0x0400;
    pub const TC: u16 = 0x0200;
    pub const RD: u16 = 0x0100;
    pub const RA: u16 = 0x0080;
    pub const AD: u16 = 0x0020;
    pub const CD: u16 = 0x0010;
    pub const RCODE_MASK: u16 = 0x000F;
}

/// Transport over which a DNS message was received.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Udp,
    Tcp,
}

impl Protocol {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Udp => "UDP",
            Self::Tcp => "TCP",
        }
    }
}

/// DNS response codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rcode {
    NoError,
    FormErr,
    ServFail,
    NxDomain,
    NotImp,
    Refused,
    Unknown(u8),
}

impl Rcode {
    #[must_use]
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::NoError,
            1 => Self::FormErr,
            2 => Self::ServFail,
            3 => Self::NxDomain,
            4 => Self::NotImp,
            5 => Self::Refused,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub fn as_u8(self) -> u8 {
        match self {
            Self::NoError => 0,
            Self::FormErr => 1,
            Self::ServFail => 2,
            Self::NxDomain => 3,
            Self::NotImp => 4,
            Self::Refused => 5,
            Self::Unknown(value) => value,
        }
    }

    /// RFC DNS mnemonic used by query logs and traffic statistics.
    #[must_use]
    pub fn mnemonic(self) -> &'static str {
        match self {
            Self::NoError => "NOERROR",
            Self::FormErr => "FORMERR",
            Self::ServFail => "SERVFAIL",
            Self::NxDomain => "NXDOMAIN",
            Self::NotImp => "NOTIMP",
            Self::Refused => "REFUSED",
            Self::Unknown(_) => "UNKNOWN",
        }
    }
}

/// Errors produced while parsing or encoding a DNS packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketError {
    Truncated,
    InvalidName,
    InvalidLabel,
    MessageTooLarge,
    TooManyRecords,
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("truncated DNS packet"),
            Self::InvalidName => f.write_str("invalid DNS name"),
            Self::InvalidLabel => f.write_str("invalid DNS label"),
            Self::MessageTooLarge => f.write_str("DNS message too large"),
            Self::TooManyRecords => f.write_str("too many DNS records"),
        }
    }
}

impl std::error::Error for PacketError {}

/// DNS message header.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Header {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl Header {
    pub fn parse(buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < HEADER_LEN {
            return Err(PacketError::Truncated);
        }
        Ok(Self {
            id: u16::from_be_bytes([buf[0], buf[1]]),
            flags: u16::from_be_bytes([buf[2], buf[3]]),
            qdcount: u16::from_be_bytes([buf[4], buf[5]]),
            ancount: u16::from_be_bytes([buf[6], buf[7]]),
            nscount: u16::from_be_bytes([buf[8], buf[9]]),
            arcount: u16::from_be_bytes([buf[10], buf[11]]),
        })
    }

    fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.id.to_be_bytes());
        out.extend_from_slice(&self.flags.to_be_bytes());
        out.extend_from_slice(&self.qdcount.to_be_bytes());
        out.extend_from_slice(&self.ancount.to_be_bytes());
        out.extend_from_slice(&self.nscount.to_be_bytes());
        out.extend_from_slice(&self.arcount.to_be_bytes());
    }

    #[must_use]
    pub fn is_response(&self) -> bool {
        self.flags & flags::QR != 0
    }

    #[must_use]
    pub fn opcode(&self) -> u8 {
        ((self.flags & flags::OPCODE_MASK) >> 11) as u8
    }

    #[must_use]
    pub fn authoritative(&self) -> bool {
        self.flags & flags::AA != 0
    }

    pub fn set_authoritative(&mut self, value: bool) {
        self.set_flag(flags::AA, value);
    }

    #[must_use]
    pub fn truncated(&self) -> bool {
        self.flags & flags::TC != 0
    }

    pub fn set_truncated(&mut self, value: bool) {
        self.set_flag(flags::TC, value);
    }

    #[must_use]
    pub fn recursion_desired(&self) -> bool {
        self.flags & flags::RD != 0
    }

    pub fn set_recursion_desired(&mut self, value: bool) {
        self.set_flag(flags::RD, value);
    }

    #[must_use]
    pub fn recursion_available(&self) -> bool {
        self.flags & flags::RA != 0
    }

    pub fn set_recursion_available(&mut self, value: bool) {
        self.set_flag(flags::RA, value);
    }

    #[must_use]
    pub fn rcode(&self) -> Rcode {
        Rcode::from_u8((self.flags & flags::RCODE_MASK) as u8)
    }

    pub fn set_rcode(&mut self, rcode: Rcode) {
        self.flags = (self.flags & !flags::RCODE_MASK) | u16::from(rcode.as_u8());
    }

    fn set_flag(&mut self, flag: u16, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }
}

/// A question section entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub name: String,
    pub qtype: u16,
    pub qclass: u16,
}

impl Question {
    pub fn parse(buf: &[u8], pos: &mut usize) -> Result<Self, PacketError> {
        let (name, next) = parse_qname(buf, *pos).ok_or(PacketError::InvalidName)?;
        if next + 4 > buf.len() {
            return Err(PacketError::Truncated);
        }
        let qtype = u16::from_be_bytes([buf[next], buf[next + 1]]);
        let qclass = u16::from_be_bytes([buf[next + 2], buf[next + 3]]);
        *pos = next + 4;
        Ok(Self {
            name,
            qtype,
            qclass,
        })
    }

    fn encode(
        &self,
        out: &mut Vec<u8>,
        compression: &mut HashMap<String, u16>,
    ) -> Result<(), PacketError> {
        encode_name(&self.name, out, compression)?;
        out.extend_from_slice(&self.qtype.to_be_bytes());
        out.extend_from_slice(&self.qclass.to_be_bytes());
        Ok(())
    }
}

/// A resource record with raw RDATA bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub rtype: u16,
    pub class: u16,
    pub ttl: u32,
    pub rdata: Vec<u8>,
    /// Absolute RDATA offset in the parsed message. Zero for records built locally.
    pub rdata_offset: usize,
}

impl Record {
    pub fn parse(buf: &[u8], pos: &mut usize) -> Result<Self, PacketError> {
        let (name, next) = parse_qname(buf, *pos).ok_or(PacketError::InvalidName)?;
        if next + 10 > buf.len() {
            return Err(PacketError::Truncated);
        }
        let rtype = u16::from_be_bytes([buf[next], buf[next + 1]]);
        let class = u16::from_be_bytes([buf[next + 2], buf[next + 3]]);
        let ttl = u32::from_be_bytes([buf[next + 4], buf[next + 5], buf[next + 6], buf[next + 7]]);
        let rdlen = u16::from_be_bytes([buf[next + 8], buf[next + 9]]) as usize;
        let rdata_offset = next + 10;
        let end = rdata_offset
            .checked_add(rdlen)
            .ok_or(PacketError::Truncated)?;
        if end > buf.len() {
            return Err(PacketError::Truncated);
        }
        *pos = end;
        Ok(Self {
            name,
            rtype,
            class,
            ttl,
            rdata: buf[rdata_offset..end].to_vec(),
            rdata_offset,
        })
    }

    fn encode(
        &self,
        out: &mut Vec<u8>,
        compression: &mut HashMap<String, u16>,
    ) -> Result<(), PacketError> {
        encode_name(&self.name, out, compression)?;
        out.extend_from_slice(&self.rtype.to_be_bytes());
        out.extend_from_slice(&self.class.to_be_bytes());
        out.extend_from_slice(&self.ttl.to_be_bytes());
        let rdlen = u16::try_from(self.rdata.len()).map_err(|_| PacketError::MessageTooLarge)?;
        out.extend_from_slice(&rdlen.to_be_bytes());
        out.extend_from_slice(&self.rdata);
        Ok(())
    }
}

/// EDNS(0) OPT pseudo-record contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edns {
    pub udp_payload_size: u16,
    pub extended_rcode: u8,
    pub version: u8,
    pub flags: u16,
    pub options: Vec<u8>,
}

impl Edns {
    fn from_record(record: &Record) -> Self {
        Self {
            udp_payload_size: record.class,
            extended_rcode: (record.ttl >> 24) as u8,
            version: (record.ttl >> 16) as u8,
            flags: record.ttl as u16,
            options: record.rdata.clone(),
        }
    }

    fn to_record(&self) -> Record {
        let ttl = (u32::from(self.extended_rcode) << 24)
            | (u32::from(self.version) << 16)
            | u32::from(self.flags);
        Record {
            name: String::new(),
            rtype: TYPE_OPT,
            class: self.udp_payload_size,
            ttl,
            rdata: self.options.clone(),
            rdata_offset: 0,
        }
    }

    /// Build the response OPT record for a query OPT record.
    #[must_use]
    pub fn response_to(query: &Self, server_payload_size: u16) -> Self {
        Self {
            udp_payload_size: server_payload_size.max(CLASSIC_UDP_PAYLOAD as u16),
            extended_rcode: 0,
            version: 0,
            flags: query.flags & 0x8000,
            options: Vec::new(),
        }
    }
}

/// A complete DNS message.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub additionals: Vec<Record>,
    pub edns: Option<Edns>,
}

impl Message {
    /// Build a standard recursive query for tests and internal callers.
    #[must_use]
    pub fn query(id: u16, name: impl Into<String>, qtype: u16) -> Self {
        Self {
            header: Header {
                id,
                flags: flags::RD,
                qdcount: 1,
                ..Header::default()
            },
            questions: vec![Question {
                name: name.into(),
                qtype,
                qclass: 1,
            }],
            ..Self::default()
        }
    }

    /// Create a response skeleton that echoes the query ID, opcode, RD bit, and question.
    #[must_use]
    pub fn response_to(query: &Self) -> Self {
        let mut header = Header {
            id: query.header.id,
            flags: flags::QR | (query.header.flags & (flags::OPCODE_MASK | flags::RD)),
            ..Header::default()
        };
        header.set_rcode(Rcode::NoError);
        Self {
            header,
            questions: query.questions.clone(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
            edns: query.edns.as_ref().map(|edns| Edns {
                udp_payload_size: edns.udp_payload_size,
                extended_rcode: 0,
                version: 0,
                flags: edns.flags & 0x8000,
                options: Vec::new(),
            }),
        }
    }

    pub fn parse(buf: &[u8]) -> Result<Self, PacketError> {
        let header = Header::parse(buf)?;
        let mut pos = HEADER_LEN;

        let mut questions = Vec::with_capacity(usize::from(header.qdcount).min(16));
        for _ in 0..header.qdcount {
            questions.push(Question::parse(buf, &mut pos)?);
        }

        let answers = parse_records(buf, &mut pos, header.ancount)?;
        let authorities = parse_records(buf, &mut pos, header.nscount)?;

        let mut additionals = Vec::with_capacity(usize::from(header.arcount).min(16));
        let mut edns = None;
        for _ in 0..header.arcount {
            let record = Record::parse(buf, &mut pos)?;
            if record.rtype == TYPE_OPT {
                if edns.is_none() {
                    edns = Some(Edns::from_record(&record));
                }
            } else {
                additionals.push(record);
            }
        }

        Ok(Self {
            header,
            questions,
            answers,
            authorities,
            additionals,
            edns,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, PacketError> {
        let qdcount = count(self.questions.len())?;
        let ancount = count(self.answers.len())?;
        let nscount = count(self.authorities.len())?;
        let arcount = count(self.additionals.len() + usize::from(self.edns.is_some()))?;

        let header = Header {
            qdcount,
            ancount,
            nscount,
            arcount,
            ..self.header
        };

        let mut out = Vec::with_capacity(CLASSIC_UDP_PAYLOAD);
        header.write(&mut out);
        let mut compression = HashMap::new();

        for question in &self.questions {
            question.encode(&mut out, &mut compression)?;
        }
        for record in &self.answers {
            record.encode(&mut out, &mut compression)?;
        }
        for record in &self.authorities {
            record.encode(&mut out, &mut compression)?;
        }
        for record in &self.additionals {
            record.encode(&mut out, &mut compression)?;
        }
        if let Some(edns) = &self.edns {
            edns.to_record().encode(&mut out, &mut compression)?;
        }

        Ok(out)
    }

    /// Encode for UDP, truncating the data sections if the message exceeds `max_payload`.
    pub fn encode_udp(&self, max_payload: usize) -> Result<Vec<u8>, PacketError> {
        let bytes = self.encode()?;
        if bytes.len() <= max_payload {
            return Ok(bytes);
        }

        let mut truncated = self.clone();
        truncated.header.set_truncated(true);
        truncated.answers.clear();
        truncated.authorities.clear();
        truncated.additionals.clear();
        truncated.encode()
    }
}

fn parse_records(buf: &[u8], pos: &mut usize, count: u16) -> Result<Vec<Record>, PacketError> {
    let mut records = Vec::with_capacity(usize::from(count).min(16));
    for _ in 0..count {
        records.push(Record::parse(buf, pos)?);
    }
    Ok(records)
}

fn count(value: usize) -> Result<u16, PacketError> {
    u16::try_from(value).map_err(|_| PacketError::TooManyRecords)
}

fn encode_name(
    name: &str,
    out: &mut Vec<u8>,
    compression: &mut HashMap<String, u16>,
) -> Result<(), PacketError> {
    let trimmed = name.trim_end_matches('.');
    if trimmed.is_empty() {
        out.push(0);
        return Ok(());
    }

    let labels: Vec<&str> = trimmed
        .split('.')
        .filter(|label| !label.is_empty())
        .collect();
    if labels.len() > MAX_LABELS {
        return Err(PacketError::InvalidName);
    }

    let start = out.len();
    for (index, label) in labels.iter().enumerate() {
        let suffix = labels[index..].join(".");
        if let Some(&offset) = compression.get(&suffix)
            && offset <= 0x3FFF
        {
            out.extend_from_slice(&(0xC000 | offset).to_be_bytes());
            return Ok(());
        }

        let offset = u16::try_from(out.len()).map_err(|_| PacketError::MessageTooLarge)?;
        if offset <= 0x3FFF {
            compression.insert(suffix, offset);
        }

        let len = u8::try_from(label.len()).map_err(|_| PacketError::InvalidLabel)?;
        if len > 63 {
            return Err(PacketError::InvalidLabel);
        }
        out.push(len);
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0);

    if out.len() - start > MAX_NAME_LEN {
        return Err(PacketError::InvalidName);
    }
    Ok(())
}
