//! DNS record types and resource data.
//!
//! These types back the in-memory zone store and are independent of the wire codec.
//! Names are stored as canonical lowercase fully qualified strings with a trailing dot.
//!
//! Common record types have typed resource data.
//! Every other type in the IANA registry can be stored with the RFC 3597 presentation format (`\# <length> <hex>`),
//! which is also how unknown types are displayed.

use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use crate::dns::packet;
use crate::dns::wire::encode_name_labels_vec;

/// DNS resource record types supported by the zone store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RecordType {
    A,
    Ns,
    Md,
    Mf,
    Cname,
    Soa,
    Mb,
    Mg,
    Mr,
    Null,
    Wks,
    Ptr,
    Hinfo,
    Minfo,
    Mx,
    Txt,
    Rp,
    Afsdb,
    X25,
    Isdn,
    Rt,
    Nsap,
    NsapPtr,
    Sig,
    Key,
    Px,
    Gpos,
    Aaaa,
    Loc,
    Nxt,
    Eid,
    Nimloc,
    Srv,
    Atma,
    Naptr,
    Kx,
    Cert,
    A6,
    Dname,
    Sink,
    Opt,
    Apl,
    Ds,
    Sshfp,
    Ipseckey,
    Rrsig,
    Nsec,
    Dnskey,
    Dhcid,
    Nsec3,
    Nsec3param,
    Tlsa,
    Smimea,
    Delegparam,
    Hip,
    Ninfo,
    Rkey,
    Talink,
    Cds,
    Cdnskey,
    Openpgpkey,
    Csync,
    Zonemd,
    Svcb,
    Https,
    Dsync,
    Hhit,
    Brid,
    Unece,
    Iso,
    Spf,
    Uinfo,
    Uid,
    Gid,
    Unspec,
    Nid,
    L32,
    L64,
    Lp,
    Eui48,
    Eui64,
    Nxname,
    Tkey,
    Tsig,
    Ixfr,
    Axfr,
    Mailb,
    Maila,
    Any,
    Uri,
    Caa,
    Avc,
    Doa,
    Amtrelay,
    Resinfo,
    Wallet,
    Cla,
    Ipn,
    Ta,
    Dlv,
    Deleg,
    Aname,
    Unknown(u16),
}

impl RecordType {
    #[must_use]
    pub const fn from_u16(value: u16) -> Self {
        match value {
            1 => Self::A,
            2 => Self::Ns,
            3 => Self::Md,
            4 => Self::Mf,
            5 => Self::Cname,
            6 => Self::Soa,
            7 => Self::Mb,
            8 => Self::Mg,
            9 => Self::Mr,
            10 => Self::Null,
            11 => Self::Wks,
            12 => Self::Ptr,
            13 => Self::Hinfo,
            14 => Self::Minfo,
            15 => Self::Mx,
            16 => Self::Txt,
            17 => Self::Rp,
            18 => Self::Afsdb,
            19 => Self::X25,
            20 => Self::Isdn,
            21 => Self::Rt,
            22 => Self::Nsap,
            23 => Self::NsapPtr,
            24 => Self::Sig,
            25 => Self::Key,
            26 => Self::Px,
            27 => Self::Gpos,
            28 => Self::Aaaa,
            29 => Self::Loc,
            30 => Self::Nxt,
            31 => Self::Eid,
            32 => Self::Nimloc,
            33 => Self::Srv,
            34 => Self::Atma,
            35 => Self::Naptr,
            36 => Self::Kx,
            37 => Self::Cert,
            38 => Self::A6,
            39 => Self::Dname,
            40 => Self::Sink,
            41 => Self::Opt,
            42 => Self::Apl,
            43 => Self::Ds,
            44 => Self::Sshfp,
            45 => Self::Ipseckey,
            46 => Self::Rrsig,
            47 => Self::Nsec,
            48 => Self::Dnskey,
            49 => Self::Dhcid,
            50 => Self::Nsec3,
            51 => Self::Nsec3param,
            52 => Self::Tlsa,
            53 => Self::Smimea,
            54 => Self::Delegparam,
            55 => Self::Hip,
            56 => Self::Ninfo,
            57 => Self::Rkey,
            58 => Self::Talink,
            59 => Self::Cds,
            60 => Self::Cdnskey,
            61 => Self::Openpgpkey,
            62 => Self::Csync,
            63 => Self::Zonemd,
            64 => Self::Svcb,
            65 => Self::Https,
            66 => Self::Dsync,
            67 => Self::Hhit,
            68 => Self::Brid,
            69 => Self::Unece,
            70 => Self::Iso,
            99 => Self::Spf,
            100 => Self::Uinfo,
            101 => Self::Uid,
            102 => Self::Gid,
            103 => Self::Unspec,
            104 => Self::Nid,
            105 => Self::L32,
            106 => Self::L64,
            107 => Self::Lp,
            108 => Self::Eui48,
            109 => Self::Eui64,
            128 => Self::Nxname,
            249 => Self::Tkey,
            250 => Self::Tsig,
            251 => Self::Ixfr,
            252 => Self::Axfr,
            253 => Self::Mailb,
            254 => Self::Maila,
            255 => Self::Any,
            256 => Self::Uri,
            257 => Self::Caa,
            258 => Self::Avc,
            259 => Self::Doa,
            260 => Self::Amtrelay,
            261 => Self::Resinfo,
            262 => Self::Wallet,
            263 => Self::Cla,
            264 => Self::Ipn,
            32768 => Self::Ta,
            32769 => Self::Dlv,
            61440 => Self::Deleg,
            65305 => Self::Aname,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn as_u16(self) -> u16 {
        match self {
            Self::A => 1,
            Self::Ns => 2,
            Self::Md => 3,
            Self::Mf => 4,
            Self::Cname => 5,
            Self::Soa => 6,
            Self::Mb => 7,
            Self::Mg => 8,
            Self::Mr => 9,
            Self::Null => 10,
            Self::Wks => 11,
            Self::Ptr => 12,
            Self::Hinfo => 13,
            Self::Minfo => 14,
            Self::Mx => 15,
            Self::Txt => 16,
            Self::Rp => 17,
            Self::Afsdb => 18,
            Self::X25 => 19,
            Self::Isdn => 20,
            Self::Rt => 21,
            Self::Nsap => 22,
            Self::NsapPtr => 23,
            Self::Sig => 24,
            Self::Key => 25,
            Self::Px => 26,
            Self::Gpos => 27,
            Self::Aaaa => 28,
            Self::Loc => 29,
            Self::Nxt => 30,
            Self::Eid => 31,
            Self::Nimloc => 32,
            Self::Srv => 33,
            Self::Atma => 34,
            Self::Naptr => 35,
            Self::Kx => 36,
            Self::Cert => 37,
            Self::A6 => 38,
            Self::Dname => 39,
            Self::Sink => 40,
            Self::Opt => 41,
            Self::Apl => 42,
            Self::Ds => 43,
            Self::Sshfp => 44,
            Self::Ipseckey => 45,
            Self::Rrsig => 46,
            Self::Nsec => 47,
            Self::Dnskey => 48,
            Self::Dhcid => 49,
            Self::Nsec3 => 50,
            Self::Nsec3param => 51,
            Self::Tlsa => 52,
            Self::Smimea => 53,
            Self::Delegparam => 54,
            Self::Hip => 55,
            Self::Ninfo => 56,
            Self::Rkey => 57,
            Self::Talink => 58,
            Self::Cds => 59,
            Self::Cdnskey => 60,
            Self::Openpgpkey => 61,
            Self::Csync => 62,
            Self::Zonemd => 63,
            Self::Svcb => 64,
            Self::Https => 65,
            Self::Dsync => 66,
            Self::Hhit => 67,
            Self::Brid => 68,
            Self::Unece => 69,
            Self::Iso => 70,
            Self::Spf => 99,
            Self::Uinfo => 100,
            Self::Uid => 101,
            Self::Gid => 102,
            Self::Unspec => 103,
            Self::Nid => 104,
            Self::L32 => 105,
            Self::L64 => 106,
            Self::Lp => 107,
            Self::Eui48 => 108,
            Self::Eui64 => 109,
            Self::Nxname => 128,
            Self::Tkey => 249,
            Self::Tsig => 250,
            Self::Ixfr => 251,
            Self::Axfr => 252,
            Self::Mailb => 253,
            Self::Maila => 254,
            Self::Any => 255,
            Self::Uri => 256,
            Self::Caa => 257,
            Self::Avc => 258,
            Self::Doa => 259,
            Self::Amtrelay => 260,
            Self::Resinfo => 261,
            Self::Wallet => 262,
            Self::Cla => 263,
            Self::Ipn => 264,
            Self::Ta => 32768,
            Self::Dlv => 32769,
            Self::Deleg => 61440,
            Self::Aname => 65305,
            Self::Unknown(value) => value,
        }
    }

    /// Return `true` when the type is a query-only pseudo type.
    #[must_use]
    pub const fn is_meta(self) -> bool {
        matches!(
            self,
            Self::Opt
                | Self::Tkey
                | Self::Tsig
                | Self::Ixfr
                | Self::Axfr
                | Self::Mailb
                | Self::Maila
                | Self::Any
        )
    }
}

/// Record type names that can be managed in a zone.
pub const RECORD_TYPES: &[&str] = &[
    "A",
    "A6",
    "AAAA",
    "AFSDB",
    "AMTRELAY",
    "ANAME",
    "APL",
    "ATMA",
    "AVC",
    "BRID",
    "CAA",
    "CDNSKEY",
    "CDS",
    "CERT",
    "CLA",
    "CNAME",
    "CSYNC",
    "DELEG",
    "DELEGPARAM",
    "DHCID",
    "DLV",
    "DNAME",
    "DNSKEY",
    "DOA",
    "DS",
    "DSYNC",
    "EID",
    "EUI48",
    "EUI64",
    "GPOS",
    "HHIT",
    "HINFO",
    "HIP",
    "HTTPS",
    "IPN",
    "IPSECKEY",
    "ISDN",
    "ISO",
    "KEY",
    "KX",
    "L32",
    "L64",
    "LOC",
    "LP",
    "MB",
    "MD",
    "MF",
    "MG",
    "MINFO",
    "MR",
    "MX",
    "NAPTR",
    "NID",
    "NIMLOC",
    "NINFO",
    "NS",
    "NSAP",
    "NSAP-PTR",
    "NSEC",
    "NSEC3",
    "NSEC3PARAM",
    "NULL",
    "NXNAME",
    "NXT",
    "OPENPGPKEY",
    "PTR",
    "PX",
    "RESINFO",
    "RKEY",
    "RP",
    "RRSIG",
    "RT",
    "SIG",
    "SINK",
    "SMIMEA",
    "SOA",
    "SPF",
    "SRV",
    "SSHFP",
    "SVCB",
    "TA",
    "TALINK",
    "TLSA",
    "TXT",
    "UID",
    "UINFO",
    "UNECE",
    "UNSPEC",
    "URI",
    "WALLET",
    "WKS",
    "X25",
    "ZONEMD",
];

impl FromStr for RecordType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let upper = value.trim().to_ascii_uppercase();
        let record_type = match upper.as_str() {
            "A" => Self::A,
            "NS" => Self::Ns,
            "MD" => Self::Md,
            "MF" => Self::Mf,
            "CNAME" => Self::Cname,
            "SOA" => Self::Soa,
            "MB" => Self::Mb,
            "MG" => Self::Mg,
            "MR" => Self::Mr,
            "NULL" => Self::Null,
            "WKS" => Self::Wks,
            "PTR" => Self::Ptr,
            "HINFO" => Self::Hinfo,
            "MINFO" => Self::Minfo,
            "MX" => Self::Mx,
            "TXT" => Self::Txt,
            "RP" => Self::Rp,
            "AFSDB" => Self::Afsdb,
            "X25" => Self::X25,
            "ISDN" => Self::Isdn,
            "RT" => Self::Rt,
            "NSAP" => Self::Nsap,
            "NSAP-PTR" => Self::NsapPtr,
            "SIG" => Self::Sig,
            "KEY" => Self::Key,
            "PX" => Self::Px,
            "GPOS" => Self::Gpos,
            "AAAA" => Self::Aaaa,
            "LOC" => Self::Loc,
            "NXT" => Self::Nxt,
            "EID" => Self::Eid,
            "NIMLOC" => Self::Nimloc,
            "SRV" => Self::Srv,
            "ATMA" => Self::Atma,
            "NAPTR" => Self::Naptr,
            "KX" => Self::Kx,
            "CERT" => Self::Cert,
            "A6" => Self::A6,
            "DNAME" => Self::Dname,
            "SINK" => Self::Sink,
            "OPT" => Self::Opt,
            "APL" => Self::Apl,
            "DS" => Self::Ds,
            "SSHFP" => Self::Sshfp,
            "IPSECKEY" => Self::Ipseckey,
            "RRSIG" => Self::Rrsig,
            "NSEC" => Self::Nsec,
            "DNSKEY" => Self::Dnskey,
            "DHCID" => Self::Dhcid,
            "NSEC3" => Self::Nsec3,
            "NSEC3PARAM" => Self::Nsec3param,
            "TLSA" => Self::Tlsa,
            "SMIMEA" => Self::Smimea,
            "DELEGPARAM" => Self::Delegparam,
            "HIP" => Self::Hip,
            "NINFO" => Self::Ninfo,
            "RKEY" => Self::Rkey,
            "TALINK" => Self::Talink,
            "CDS" => Self::Cds,
            "CDNSKEY" => Self::Cdnskey,
            "OPENPGPKEY" => Self::Openpgpkey,
            "CSYNC" => Self::Csync,
            "ZONEMD" => Self::Zonemd,
            "SVCB" => Self::Svcb,
            "HTTPS" => Self::Https,
            "DSYNC" => Self::Dsync,
            "HHIT" => Self::Hhit,
            "BRID" => Self::Brid,
            "UNECE" => Self::Unece,
            "ISO" => Self::Iso,
            "SPF" => Self::Spf,
            "UINFO" => Self::Uinfo,
            "UID" => Self::Uid,
            "GID" => Self::Gid,
            "UNSPEC" => Self::Unspec,
            "NID" => Self::Nid,
            "L32" => Self::L32,
            "L64" => Self::L64,
            "LP" => Self::Lp,
            "EUI48" => Self::Eui48,
            "EUI64" => Self::Eui64,
            "NXNAME" => Self::Nxname,
            "TKEY" => Self::Tkey,
            "TSIG" => Self::Tsig,
            "IXFR" => Self::Ixfr,
            "AXFR" => Self::Axfr,
            "MAILB" => Self::Mailb,
            "MAILA" => Self::Maila,
            "ANY" => Self::Any,
            "URI" => Self::Uri,
            "CAA" => Self::Caa,
            "AVC" => Self::Avc,
            "DOA" => Self::Doa,
            "AMTRELAY" => Self::Amtrelay,
            "RESINFO" => Self::Resinfo,
            "WALLET" => Self::Wallet,
            "CLA" => Self::Cla,
            "IPN" => Self::Ipn,
            "TA" => Self::Ta,
            "DLV" => Self::Dlv,
            "DELEG" => Self::Deleg,
            "ANAME" => Self::Aname,
            other => {
                return other
                    .strip_prefix("TYPE")
                    .and_then(|number| number.parse::<u16>().ok())
                    .map_or(Err(()), |number| Ok(Self::Unknown(number)));
            }
        };
        Ok(record_type)
    }
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A => f.write_str("A"),
            Self::Ns => f.write_str("NS"),
            Self::Md => f.write_str("MD"),
            Self::Mf => f.write_str("MF"),
            Self::Cname => f.write_str("CNAME"),
            Self::Soa => f.write_str("SOA"),
            Self::Mb => f.write_str("MB"),
            Self::Mg => f.write_str("MG"),
            Self::Mr => f.write_str("MR"),
            Self::Null => f.write_str("NULL"),
            Self::Wks => f.write_str("WKS"),
            Self::Ptr => f.write_str("PTR"),
            Self::Hinfo => f.write_str("HINFO"),
            Self::Minfo => f.write_str("MINFO"),
            Self::Mx => f.write_str("MX"),
            Self::Txt => f.write_str("TXT"),
            Self::Rp => f.write_str("RP"),
            Self::Afsdb => f.write_str("AFSDB"),
            Self::X25 => f.write_str("X25"),
            Self::Isdn => f.write_str("ISDN"),
            Self::Rt => f.write_str("RT"),
            Self::Nsap => f.write_str("NSAP"),
            Self::NsapPtr => f.write_str("NSAP-PTR"),
            Self::Sig => f.write_str("SIG"),
            Self::Key => f.write_str("KEY"),
            Self::Px => f.write_str("PX"),
            Self::Gpos => f.write_str("GPOS"),
            Self::Aaaa => f.write_str("AAAA"),
            Self::Loc => f.write_str("LOC"),
            Self::Nxt => f.write_str("NXT"),
            Self::Eid => f.write_str("EID"),
            Self::Nimloc => f.write_str("NIMLOC"),
            Self::Srv => f.write_str("SRV"),
            Self::Atma => f.write_str("ATMA"),
            Self::Naptr => f.write_str("NAPTR"),
            Self::Kx => f.write_str("KX"),
            Self::Cert => f.write_str("CERT"),
            Self::A6 => f.write_str("A6"),
            Self::Dname => f.write_str("DNAME"),
            Self::Sink => f.write_str("SINK"),
            Self::Opt => f.write_str("OPT"),
            Self::Apl => f.write_str("APL"),
            Self::Ds => f.write_str("DS"),
            Self::Sshfp => f.write_str("SSHFP"),
            Self::Ipseckey => f.write_str("IPSECKEY"),
            Self::Rrsig => f.write_str("RRSIG"),
            Self::Nsec => f.write_str("NSEC"),
            Self::Dnskey => f.write_str("DNSKEY"),
            Self::Dhcid => f.write_str("DHCID"),
            Self::Nsec3 => f.write_str("NSEC3"),
            Self::Nsec3param => f.write_str("NSEC3PARAM"),
            Self::Tlsa => f.write_str("TLSA"),
            Self::Smimea => f.write_str("SMIMEA"),
            Self::Delegparam => f.write_str("DELEGPARAM"),
            Self::Hip => f.write_str("HIP"),
            Self::Ninfo => f.write_str("NINFO"),
            Self::Rkey => f.write_str("RKEY"),
            Self::Talink => f.write_str("TALINK"),
            Self::Cds => f.write_str("CDS"),
            Self::Cdnskey => f.write_str("CDNSKEY"),
            Self::Openpgpkey => f.write_str("OPENPGPKEY"),
            Self::Csync => f.write_str("CSYNC"),
            Self::Zonemd => f.write_str("ZONEMD"),
            Self::Svcb => f.write_str("SVCB"),
            Self::Https => f.write_str("HTTPS"),
            Self::Dsync => f.write_str("DSYNC"),
            Self::Hhit => f.write_str("HHIT"),
            Self::Brid => f.write_str("BRID"),
            Self::Unece => f.write_str("UNECE"),
            Self::Iso => f.write_str("ISO"),
            Self::Spf => f.write_str("SPF"),
            Self::Uinfo => f.write_str("UINFO"),
            Self::Uid => f.write_str("UID"),
            Self::Gid => f.write_str("GID"),
            Self::Unspec => f.write_str("UNSPEC"),
            Self::Nid => f.write_str("NID"),
            Self::L32 => f.write_str("L32"),
            Self::L64 => f.write_str("L64"),
            Self::Lp => f.write_str("LP"),
            Self::Eui48 => f.write_str("EUI48"),
            Self::Eui64 => f.write_str("EUI64"),
            Self::Nxname => f.write_str("NXNAME"),
            Self::Tkey => f.write_str("TKEY"),
            Self::Tsig => f.write_str("TSIG"),
            Self::Ixfr => f.write_str("IXFR"),
            Self::Axfr => f.write_str("AXFR"),
            Self::Mailb => f.write_str("MAILB"),
            Self::Maila => f.write_str("MAILA"),
            Self::Any => f.write_str("ANY"),
            Self::Uri => f.write_str("URI"),
            Self::Caa => f.write_str("CAA"),
            Self::Avc => f.write_str("AVC"),
            Self::Doa => f.write_str("DOA"),
            Self::Amtrelay => f.write_str("AMTRELAY"),
            Self::Resinfo => f.write_str("RESINFO"),
            Self::Wallet => f.write_str("WALLET"),
            Self::Cla => f.write_str("CLA"),
            Self::Ipn => f.write_str("IPN"),
            Self::Ta => f.write_str("TA"),
            Self::Dlv => f.write_str("DLV"),
            Self::Deleg => f.write_str("DELEG"),
            Self::Aname => f.write_str("ANAME"),
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
    Dname(String),
    Ptr(String),
    Aname(String),
    Mx {
        preference: u16,
        exchange: String,
    },
    Kx {
        preference: u16,
        exchanger: String,
    },
    Txt(Vec<String>),
    Spf(Vec<String>),
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
    Hinfo {
        cpu: String,
        os: String,
    },
    Rp {
        mbox: String,
        txt: String,
    },
    Naptr {
        order: u16,
        preference: u16,
        flags: String,
        services: String,
        regexp: String,
        replacement: String,
    },
    Ds {
        rtype: RecordType,
        key_tag: u16,
        algorithm: u8,
        digest_type: u8,
        digest: Vec<u8>,
    },
    Dnskey {
        rtype: RecordType,
        flags: u16,
        protocol: u8,
        algorithm: u8,
        public_key: Vec<u8>,
    },
    Rrsig {
        type_covered: u16,
        algorithm: u8,
        labels: u8,
        original_ttl: u32,
        expiration: u32,
        inception: u32,
        key_tag: u16,
        signer_name: String,
        signature: Vec<u8>,
    },
    Tlsa {
        rtype: RecordType,
        usage: u8,
        selector: u8,
        matching_type: u8,
        certificate_association_data: Vec<u8>,
    },
    Sshfp {
        algorithm: u8,
        fingerprint_type: u8,
        fingerprint: Vec<u8>,
    },
    Svcb {
        rtype: RecordType,
        priority: u16,
        target: String,
        params: Vec<u8>,
    },
    Uri {
        priority: u16,
        weight: u16,
        target: String,
    },
    Cert {
        cert_type: u16,
        key_tag: u16,
        algorithm: u8,
        certificate: Vec<u8>,
    },
    Eui48([u8; 6]),
    Eui64([u8; 8]),
    /// RFC 3597 opaque resource data. Used for every type without a typed encoder.
    Unknown {
        rtype: RecordType,
        data: Vec<u8>,
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
            Self::Dname(_) => RecordType::Dname,
            Self::Ptr(_) => RecordType::Ptr,
            Self::Aname(_) => RecordType::Aname,
            Self::Mx { .. } => RecordType::Mx,
            Self::Kx { .. } => RecordType::Kx,
            Self::Txt(_) => RecordType::Txt,
            Self::Spf(_) => RecordType::Spf,
            Self::Soa { .. } => RecordType::Soa,
            Self::Srv { .. } => RecordType::Srv,
            Self::Caa { .. } => RecordType::Caa,
            Self::Hinfo { .. } => RecordType::Hinfo,
            Self::Rp { .. } => RecordType::Rp,
            Self::Naptr { .. } => RecordType::Naptr,
            Self::Ds { rtype, .. } => *rtype,
            Self::Dnskey { rtype, .. } => *rtype,
            Self::Rrsig { .. } => RecordType::Rrsig,
            Self::Tlsa { rtype, .. } => *rtype,
            Self::Sshfp { .. } => RecordType::Sshfp,
            Self::Svcb { rtype, .. } => *rtype,
            Self::Uri { .. } => RecordType::Uri,
            Self::Cert { .. } => RecordType::Cert,
            Self::Eui48(_) => RecordType::Eui48,
            Self::Eui64(_) => RecordType::Eui64,
            Self::Unknown { rtype, .. } => *rtype,
        }
    }

    /// Parse record data from its presentation format.
    ///
    /// Types without a typed parser accept the RFC 3597 generic format (`\# <length> <hex>`).
    /// The generic format is also accepted for typed types, in which case the data is stored opaquely.
    #[must_use]
    pub fn parse(rtype: RecordType, data: &str) -> Option<Self> {
        if rtype.is_meta() {
            return None;
        }
        if let Some(data) = parse_rfc3597(data) {
            return Some(Self::Unknown { rtype, data });
        }

        match rtype {
            RecordType::A => data.trim().parse::<Ipv4Addr>().ok().map(Self::A),
            RecordType::Aaaa => data.trim().parse::<Ipv6Addr>().ok().map(Self::Aaaa),
            RecordType::Ns => canonical_name(data).map(Self::Ns),
            RecordType::Cname => canonical_name(data).map(Self::Cname),
            RecordType::Dname => canonical_name(data).map(Self::Dname),
            RecordType::Ptr => canonical_name(data).map(Self::Ptr),
            RecordType::Aname => canonical_name(data).map(Self::Aname),
            RecordType::Mx => {
                let parts = split_presentation(data);
                if parts.len() != 2 {
                    return None;
                }
                Some(Self::Mx {
                    preference: parts[0].parse::<u16>().ok()?,
                    exchange: canonical_name(&parts[1])?,
                })
            }
            RecordType::Kx => {
                let parts = split_presentation(data);
                if parts.len() != 2 {
                    return None;
                }
                Some(Self::Kx {
                    preference: parts[0].parse::<u16>().ok()?,
                    exchanger: canonical_name(&parts[1])?,
                })
            }
            RecordType::Txt => Some(Self::Txt(parse_txt(data))),
            RecordType::Spf => Some(Self::Spf(parse_txt(data))),
            RecordType::Soa => {
                let parts = split_presentation(data);
                if parts.len() != 7 {
                    return None;
                }
                Some(Self::Soa {
                    mname: canonical_name(&parts[0])?,
                    rname: canonical_name(&parts[1])?,
                    serial: parts[2].parse::<u32>().ok()?,
                    refresh: parts[3].parse::<u32>().ok()?,
                    retry: parts[4].parse::<u32>().ok()?,
                    expire: parts[5].parse::<u32>().ok()?,
                    minimum: parts[6].parse::<u32>().ok()?,
                })
            }
            RecordType::Srv => {
                let parts = split_presentation(data);
                if parts.len() != 4 {
                    return None;
                }
                Some(Self::Srv {
                    priority: parts[0].parse::<u16>().ok()?,
                    weight: parts[1].parse::<u16>().ok()?,
                    port: parts[2].parse::<u16>().ok()?,
                    target: canonical_name(&parts[3])?,
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
            RecordType::Hinfo => {
                let parts = split_presentation(data);
                if parts.len() != 2 || !is_char_string(&parts[0]) || !is_char_string(&parts[1]) {
                    return None;
                }
                Some(Self::Hinfo {
                    cpu: parts[0].clone(),
                    os: parts[1].clone(),
                })
            }
            RecordType::Rp => {
                let parts = split_presentation(data);
                if parts.len() != 2 {
                    return None;
                }
                Some(Self::Rp {
                    mbox: canonical_name(&parts[0])?,
                    txt: canonical_name(&parts[1])?,
                })
            }
            RecordType::Naptr => {
                let parts = split_presentation(data);
                if parts.len() != 6 {
                    return None;
                }
                if !is_char_string(&parts[2])
                    || !is_char_string(&parts[3])
                    || !is_char_string(&parts[4])
                {
                    return None;
                }
                Some(Self::Naptr {
                    order: parts[0].parse::<u16>().ok()?,
                    preference: parts[1].parse::<u16>().ok()?,
                    flags: parts[2].clone(),
                    services: parts[3].clone(),
                    regexp: parts[4].clone(),
                    replacement: canonical_name(&parts[5])?,
                })
            }
            RecordType::Ds | RecordType::Cds => {
                let parts = split_presentation(data);
                if parts.len() < 4 {
                    return None;
                }
                Some(Self::Ds {
                    rtype,
                    key_tag: parts[0].parse::<u16>().ok()?,
                    algorithm: parts[1].parse::<u8>().ok()?,
                    digest_type: parts[2].parse::<u8>().ok()?,
                    digest: decode_hex(&parts[3..].join(""))?,
                })
            }
            RecordType::Dnskey | RecordType::Cdnskey | RecordType::Key | RecordType::Rkey => {
                let parts = split_presentation(data);
                if parts.len() < 4 {
                    return None;
                }
                Some(Self::Dnskey {
                    rtype,
                    flags: parts[0].parse::<u16>().ok()?,
                    protocol: parts[1].parse::<u8>().ok()?,
                    algorithm: parts[2].parse::<u8>().ok()?,
                    public_key: BASE64_STANDARD.decode(parts[3..].join("")).ok()?,
                })
            }
            RecordType::Rrsig => {
                let parts = split_presentation(data);
                if parts.len() < 9 {
                    return None;
                }
                Some(Self::Rrsig {
                    type_covered: RecordType::from_str(&parts[0]).ok()?.as_u16(),
                    algorithm: parts[1].parse::<u8>().ok()?,
                    labels: parts[2].parse::<u8>().ok()?,
                    original_ttl: parts[3].parse::<u32>().ok()?,
                    expiration: parse_dns_time(&parts[4])?,
                    inception: parse_dns_time(&parts[5])?,
                    key_tag: parts[6].parse::<u16>().ok()?,
                    signer_name: canonical_name(&parts[7])?,
                    signature: BASE64_STANDARD.decode(parts[8..].join("")).ok()?,
                })
            }
            RecordType::Tlsa | RecordType::Smimea => {
                let parts = split_presentation(data);
                if parts.len() < 4 {
                    return None;
                }
                Some(Self::Tlsa {
                    rtype,
                    usage: parts[0].parse::<u8>().ok()?,
                    selector: parts[1].parse::<u8>().ok()?,
                    matching_type: parts[2].parse::<u8>().ok()?,
                    certificate_association_data: decode_hex(&parts[3..].join(""))?,
                })
            }
            RecordType::Sshfp => {
                let parts = split_presentation(data);
                if parts.len() < 3 {
                    return None;
                }
                Some(Self::Sshfp {
                    algorithm: parts[0].parse::<u8>().ok()?,
                    fingerprint_type: parts[1].parse::<u8>().ok()?,
                    fingerprint: decode_hex(&parts[2..].join(""))?,
                })
            }
            RecordType::Svcb | RecordType::Https => {
                let parts = split_presentation(data);
                if parts.len() < 2 {
                    return None;
                }
                let params = parse_svcb_params(&parts[2..])?;
                Some(Self::Svcb {
                    rtype,
                    priority: parts[0].parse::<u16>().ok()?,
                    target: canonical_name(&parts[1])?,
                    params,
                })
            }
            RecordType::Uri => {
                let parts = split_presentation(data);
                if parts.len() != 3 {
                    return None;
                }
                Some(Self::Uri {
                    priority: parts[0].parse::<u16>().ok()?,
                    weight: parts[1].parse::<u16>().ok()?,
                    target: parts[2].clone(),
                })
            }
            RecordType::Cert => {
                let parts = split_presentation(data);
                if parts.len() < 4 {
                    return None;
                }
                Some(Self::Cert {
                    cert_type: parts[0].parse::<u16>().ok()?,
                    key_tag: parts[1].parse::<u16>().ok()?,
                    algorithm: parts[2].parse::<u8>().ok()?,
                    certificate: BASE64_STANDARD.decode(parts[3..].join("")).ok()?,
                })
            }
            RecordType::Eui48 => parse_eui48(data).map(Self::Eui48),
            RecordType::Eui64 => parse_eui64(data).map(Self::Eui64),
            _ => None,
        }
    }

    /// Encode RDATA to DNS wire format. Names are uncompressed.
    #[must_use]
    pub fn to_wire(&self) -> Vec<u8> {
        match self {
            Self::A(address) => address.octets().to_vec(),
            Self::Aaaa(address) => address.octets().to_vec(),
            Self::Ns(name)
            | Self::Cname(name)
            | Self::Dname(name)
            | Self::Ptr(name)
            | Self::Aname(name) => encode_name_labels_vec(name),
            Self::Mx {
                preference,
                exchange,
            } => {
                let mut out = preference.to_be_bytes().to_vec();
                out.extend_from_slice(&encode_name_labels_vec(exchange));
                out
            }
            Self::Kx {
                preference,
                exchanger,
            } => {
                let mut out = preference.to_be_bytes().to_vec();
                out.extend_from_slice(&encode_name_labels_vec(exchanger));
                out
            }
            Self::Txt(values) | Self::Spf(values) => encode_text(values),
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
            Self::Hinfo { cpu, os } => {
                let mut out = Vec::new();
                encode_char_string(&mut out, cpu);
                encode_char_string(&mut out, os);
                out
            }
            Self::Rp { mbox, txt } => {
                let mut out = encode_name_labels_vec(mbox);
                out.extend_from_slice(&encode_name_labels_vec(txt));
                out
            }
            Self::Naptr {
                order,
                preference,
                flags,
                services,
                regexp,
                replacement,
            } => {
                let mut out = order.to_be_bytes().to_vec();
                out.extend_from_slice(&preference.to_be_bytes());
                encode_char_string(&mut out, flags);
                encode_char_string(&mut out, services);
                encode_char_string(&mut out, regexp);
                out.extend_from_slice(&encode_name_labels_vec(replacement));
                out
            }
            Self::Ds {
                key_tag,
                algorithm,
                digest_type,
                digest,
                ..
            } => {
                let mut out = key_tag.to_be_bytes().to_vec();
                out.push(*algorithm);
                out.push(*digest_type);
                out.extend_from_slice(digest);
                out
            }
            Self::Dnskey {
                flags,
                protocol,
                algorithm,
                public_key,
                ..
            } => {
                let mut out = flags.to_be_bytes().to_vec();
                out.push(*protocol);
                out.push(*algorithm);
                out.extend_from_slice(public_key);
                out
            }
            Self::Rrsig {
                type_covered,
                algorithm,
                labels,
                original_ttl,
                expiration,
                inception,
                key_tag,
                signer_name,
                signature,
            } => {
                let mut out = type_covered.to_be_bytes().to_vec();
                out.push(*algorithm);
                out.push(*labels);
                out.extend_from_slice(&original_ttl.to_be_bytes());
                out.extend_from_slice(&expiration.to_be_bytes());
                out.extend_from_slice(&inception.to_be_bytes());
                out.extend_from_slice(&key_tag.to_be_bytes());
                out.extend_from_slice(&encode_name_labels_vec(signer_name));
                out.extend_from_slice(signature);
                out
            }
            Self::Tlsa {
                usage,
                selector,
                matching_type,
                certificate_association_data,
                ..
            } => {
                let mut out = vec![*usage, *selector, *matching_type];
                out.extend_from_slice(certificate_association_data);
                out
            }
            Self::Sshfp {
                algorithm,
                fingerprint_type,
                fingerprint,
            } => {
                let mut out = vec![*algorithm, *fingerprint_type];
                out.extend_from_slice(fingerprint);
                out
            }
            Self::Svcb {
                priority,
                target,
                params,
                ..
            } => {
                let mut out = priority.to_be_bytes().to_vec();
                out.extend_from_slice(&encode_name_labels_vec(target));
                out.extend_from_slice(params);
                out
            }
            Self::Uri {
                priority,
                weight,
                target,
            } => {
                let mut out = priority.to_be_bytes().to_vec();
                out.extend_from_slice(&weight.to_be_bytes());
                out.extend_from_slice(target.as_bytes());
                out
            }
            Self::Cert {
                cert_type,
                key_tag,
                algorithm,
                certificate,
            } => {
                let mut out = cert_type.to_be_bytes().to_vec();
                out.extend_from_slice(&key_tag.to_be_bytes());
                out.push(*algorithm);
                out.extend_from_slice(certificate);
                out
            }
            Self::Eui48(address) => address.to_vec(),
            Self::Eui64(address) => address.to_vec(),
            Self::Unknown { data, .. } => data.clone(),
        }
    }
}

impl fmt::Display for RData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A(address) => write!(f, "{address}"),
            Self::Aaaa(address) => write!(f, "{address}"),
            Self::Ns(name)
            | Self::Cname(name)
            | Self::Dname(name)
            | Self::Ptr(name)
            | Self::Aname(name) => f.write_str(name),
            Self::Mx {
                preference,
                exchange,
            } => write!(f, "{preference} {exchange}"),
            Self::Kx {
                preference,
                exchanger,
            } => write!(f, "{preference} {exchanger}"),
            Self::Txt(values) | Self::Spf(values) => write!(f, "{}", values.join(" ")),
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
                write!(f, "{flags} {tag} \"{}\"", String::from_utf8_lossy(value))
            }
            Self::Hinfo { cpu, os } => write!(f, "\"{cpu}\" \"{os}\""),
            Self::Rp { mbox, txt } => write!(f, "{mbox} {txt}"),
            Self::Naptr {
                order,
                preference,
                flags,
                services,
                regexp,
                replacement,
            } => write!(
                f,
                "{order} {preference} \"{flags}\" \"{services}\" \"{regexp}\" {replacement}"
            ),
            Self::Ds {
                key_tag,
                algorithm,
                digest_type,
                digest,
                ..
            } => write!(
                f,
                "{key_tag} {algorithm} {digest_type} {}",
                encode_hex(digest)
            ),
            Self::Dnskey {
                flags,
                protocol,
                algorithm,
                public_key,
                ..
            } => write!(
                f,
                "{flags} {protocol} {algorithm} {}",
                BASE64_STANDARD.encode(public_key)
            ),
            Self::Rrsig {
                type_covered,
                algorithm,
                labels,
                original_ttl,
                expiration,
                inception,
                key_tag,
                signer_name,
                signature,
            } => write!(
                f,
                "{} {algorithm} {labels} {original_ttl} {expiration} {inception} {key_tag} {signer_name} {}",
                RecordType::from_u16(*type_covered),
                BASE64_STANDARD.encode(signature)
            ),
            Self::Tlsa {
                usage,
                selector,
                matching_type,
                certificate_association_data,
                ..
            } => write!(
                f,
                "{usage} {selector} {matching_type} {}",
                encode_hex(certificate_association_data)
            ),
            Self::Sshfp {
                algorithm,
                fingerprint_type,
                fingerprint,
            } => write!(
                f,
                "{algorithm} {fingerprint_type} {}",
                encode_hex(fingerprint)
            ),
            Self::Svcb {
                priority,
                target,
                params,
                ..
            } => {
                let params = format_svcb_params(params);
                if params.is_empty() {
                    write!(f, "{priority} {target}")
                } else {
                    write!(f, "{priority} {target} {params}")
                }
            }
            Self::Uri {
                priority,
                weight,
                target,
            } => write!(f, "{priority} {weight} \"{target}\""),
            Self::Cert {
                cert_type,
                key_tag,
                algorithm,
                certificate,
            } => write!(
                f,
                "{cert_type} {key_tag} {algorithm} {}",
                BASE64_STANDARD.encode(certificate)
            ),
            Self::Eui48(address) => write_eui(f, address),
            Self::Eui64(address) => write_eui(f, address),
            Self::Unknown { data, .. } => {
                if data.is_empty() {
                    f.write_str("\\# 0")
                } else {
                    write!(f, "\\# {} {}", data.len(), encode_hex(data))
                }
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

fn encode_text(values: &[String]) -> Vec<u8> {
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

fn encode_char_string(out: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    let len = bytes.len().min(usize::from(u8::MAX));
    out.push(len as u8);
    out.extend_from_slice(&bytes[..len]);
}

fn is_char_string(value: &str) -> bool {
    value.len() <= usize::from(u8::MAX)
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

fn split_presentation(data: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut chars = data.trim().chars().peekable();
    while let Some(&next) = chars.peek() {
        if next.is_whitespace() {
            chars.next();
            continue;
        }
        let mut value = String::new();
        if next == '"' {
            chars.next();
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
        } else {
            while let Some(&character) = chars.peek() {
                if character.is_whitespace() {
                    break;
                }
                value.push(character);
                chars.next();
            }
        }
        values.push(value);
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

fn parse_rfc3597(data: &str) -> Option<Vec<u8>> {
    let trimmed = data.trim_start();
    let rest = trimmed
        .strip_prefix("\\#")
        .or_else(|| trimmed.strip_prefix('#'))?;
    let mut parts = rest.split_whitespace();
    let length = parts.next()?.parse::<usize>().ok()?;
    let hex: String = parts.collect::<Vec<_>>().join("");
    let bytes = decode_hex(&hex)?;
    (bytes.len() == length).then_some(bytes)
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    let filtered: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    if !filtered.len().is_multiple_of(2) {
        return None;
    }
    let bytes = filtered.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = hex_value(bytes[index])?;
        let low = hex_value(bytes[index + 1])?;
        out.push((high << 4) | low);
        index += 2;
    }
    Some(out)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0F)]));
    }
    out
}

fn parse_dns_time(value: &str) -> Option<u32> {
    if value.len() == 14 && value.chars().all(|c| c.is_ascii_digit()) {
        let time = chrono::NaiveDateTime::parse_from_str(value, "%Y%m%d%H%M%S").ok()?;
        u32::try_from(time.and_utc().timestamp()).ok()
    } else {
        value.parse::<u32>().ok()
    }
}

fn parse_svcb_key(value: &str) -> Option<u16> {
    match value.to_ascii_lowercase().as_str() {
        "mandatory" => Some(0),
        "alpn" => Some(1),
        "no-default-alpn" => Some(2),
        "port" => Some(3),
        "ipv4hint" => Some(4),
        "ech" => Some(5),
        "ipv6hint" => Some(6),
        "dohpath" => Some(7),
        "ohttp" => Some(8),
        other => other
            .strip_prefix("key")
            .and_then(|number| number.parse::<u16>().ok()),
    }
}

fn parse_svcb_params(tokens: &[String]) -> Option<Vec<u8>> {
    let mut params: Vec<(u16, Vec<u8>)> = Vec::new();
    for token in tokens {
        let (key, value) = match token.split_once('=') {
            Some((key, value)) => (parse_svcb_key(key)?, Some(value)),
            None => (parse_svcb_key(token)?, None),
        };
        let encoded = match key {
            0 => {
                let value = value?;
                let mut out = Vec::new();
                for part in value.split(',') {
                    out.extend_from_slice(&parse_svcb_key(part)?.to_be_bytes());
                }
                out
            }
            1 => {
                let value = value?;
                let mut out = Vec::new();
                for part in value.split(',') {
                    if !is_char_string(part) {
                        return None;
                    }
                    out.push(part.len() as u8);
                    out.extend_from_slice(part.as_bytes());
                }
                out
            }
            2 | 8 => {
                if value.is_some() {
                    return None;
                }
                Vec::new()
            }
            3 => value?.parse::<u16>().ok()?.to_be_bytes().to_vec(),
            4 => {
                let mut out = Vec::new();
                for part in value?.split(',') {
                    out.extend_from_slice(&part.parse::<Ipv4Addr>().ok()?.octets());
                }
                out
            }
            5 => BASE64_STANDARD.decode(value?).ok()?,
            6 => {
                let mut out = Vec::new();
                for part in value?.split(',') {
                    out.extend_from_slice(&part.parse::<Ipv6Addr>().ok()?.octets());
                }
                out
            }
            7 => value?.as_bytes().to_vec(),
            _ => decode_hex(value?)?,
        };
        params.push((key, encoded));
    }

    params.sort_by_key(|(key, _)| *key);
    if params.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return None;
    }

    let mut out = Vec::new();
    for (key, value) in params {
        let length = u16::try_from(value.len()).ok()?;
        out.extend_from_slice(&key.to_be_bytes());
        out.extend_from_slice(&length.to_be_bytes());
        out.extend_from_slice(&value);
    }
    Some(out)
}

fn format_svcb_params(params: &[u8]) -> String {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos + 4 <= params.len() {
        let key = u16::from_be_bytes([params[pos], params[pos + 1]]);
        let length = usize::from(u16::from_be_bytes([params[pos + 2], params[pos + 3]]));
        pos += 4;
        let Some(end) = pos.checked_add(length) else {
            break;
        };
        if end > params.len() {
            break;
        }
        let value = &params[pos..end];
        pos = end;

        let rendered = match key {
            0 => {
                let mut keys = Vec::new();
                for chunk in value.as_chunks::<2>().0 {
                    let name = parse_svcb_key_name(u16::from_be_bytes(*chunk));
                    keys.push(name);
                }
                format!("mandatory={}", keys.join(","))
            }
            1 => {
                let mut values = Vec::new();
                let mut index = 0;
                while index < value.len() {
                    let length = usize::from(value[index]);
                    index += 1;
                    let Some(end) = index.checked_add(length) else {
                        break;
                    };
                    if end > value.len() {
                        break;
                    }
                    values.push(String::from_utf8_lossy(&value[index..end]).into_owned());
                    index = end;
                }
                format!("alpn={}", values.join(","))
            }
            2 => "no-default-alpn".to_string(),
            3 if value.len() == 2 => {
                format!("port={}", u16::from_be_bytes([value[0], value[1]]))
            }
            4 => {
                let mut addresses = Vec::new();
                for chunk in value.as_chunks::<4>().0 {
                    addresses.push(Ipv4Addr::from(*chunk).to_string());
                }
                format!("ipv4hint={}", addresses.join(","))
            }
            5 => format!("ech={}", BASE64_STANDARD.encode(value)),
            6 => {
                let mut addresses = Vec::new();
                for chunk in value.as_chunks::<16>().0 {
                    addresses.push(Ipv6Addr::from(*chunk).to_string());
                }
                format!("ipv6hint={}", addresses.join(","))
            }
            7 => format!("dohpath={}", String::from_utf8_lossy(value)),
            8 => "ohttp".to_string(),
            other => format!("key{other}={}", encode_hex(value)),
        };
        out.push(rendered);
    }
    out.join(" ")
}

fn parse_svcb_key_name(key: u16) -> String {
    match key {
        0 => "mandatory".to_string(),
        1 => "alpn".to_string(),
        2 => "no-default-alpn".to_string(),
        3 => "port".to_string(),
        4 => "ipv4hint".to_string(),
        5 => "ech".to_string(),
        6 => "ipv6hint".to_string(),
        7 => "dohpath".to_string(),
        8 => "ohttp".to_string(),
        other => format!("key{other}"),
    }
}

fn parse_eui48(value: &str) -> Option<[u8; 6]> {
    let normalized = value.trim().replace(':', "-");
    let mut octets = [0u8; 6];
    let mut count = 0;
    for part in normalized.split('-') {
        if count >= octets.len() {
            return None;
        }
        octets[count] = u8::from_str_radix(part, 16).ok()?;
        count += 1;
    }
    (count == octets.len()).then_some(octets)
}

fn parse_eui64(value: &str) -> Option<[u8; 8]> {
    let normalized = value.trim().replace(':', "-");
    let mut octets = [0u8; 8];
    let mut count = 0;
    for part in normalized.split('-') {
        if count >= octets.len() {
            return None;
        }
        octets[count] = u8::from_str_radix(part, 16).ok()?;
        count += 1;
    }
    (count == octets.len()).then_some(octets)
}

fn write_eui(f: &mut fmt::Formatter<'_>, address: &[u8]) -> fmt::Result {
    for (index, byte) in address.iter().enumerate() {
        if index > 0 {
            f.write_str("-")?;
        }
        write!(f, "{byte:02x}")?;
    }
    Ok(())
}
