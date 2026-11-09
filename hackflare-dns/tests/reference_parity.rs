//! Cross-check Hackflare DNS record support against the live IANA registry,
//! BIND, and ldns.
//!
//! The record list is fetched from the IANA DNS Parameters registry at test
//! time, so new assignments are picked up automatically. If the registry
//! cannot be fetched, the live checks are skipped and the local
//! `RECORD_TYPES` list is still exercised.
//!
//! Types without an explicit sample use a generated RFC 3597 sample. The
//! typed-support report is computed from the current parsers, so it always
//! reflects the code instead of a maintained list.
//!
//! Run `cargo test -p hackflare-dns --test reference_parity -- --nocapture`
//! to print the current typed-support gaps.
//!
//! The reference implementations are optional. Install `named-compilezone`
//! (BIND) or `ldns-read-zone` (ldns) to run the wire comparisons; the test
//! skips cleanly when neither tool is available.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use hackflare_dns::dns::rr::{RData, RECORD_TYPES, RecordType};
use hackflare_dns::dns::wire::encode_name_labels_vec;

const IANA_CSV_URL: &str = "https://www.iana.org/assignments/dns-parameters/dns-parameters-4.csv";

/// Query-only pseudo types in the IANA registry.
const META_CODES: &[u16] = &[41, 249, 250, 251, 252, 253, 254, 255];

/// ldns 1.9.2 serializes these types differently from BIND.
///
/// The difference is in ldns's native presentation parser, not in the wire
/// format, so the reference test skips ldns for these types and relies on BIND.
const LDNS_PRESENTATION_QUIRKS: &[&str] = &["ATMA", "GPOS", "NSAP-PTR", "RESINFO"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Support {
    /// The reference implementation is known not to accept this sample.
    Skip,
    /// Try the reference implementation, but do not fail when it rejects the sample.
    Optional,
    /// The reference implementation is known to accept this sample.
    Required,
}

/// An explicitly curated presentation sample.
struct ExplicitSample {
    rtype: &'static str,
    rdata: &'static str,
    reference_type: &'static str,
    reference_rdata: &'static str,
    wire: &'static str,
    bind: Support,
    ldns: Support,
}

/// A sample after defaults have been filled in for unlisted record types.
struct Sample {
    rtype: String,
    code: u16,
    rdata: String,
    reference_type: String,
    reference_rdata: String,
    wire: String,
    bind: Support,
    ldns: Support,
}

const EXPLICIT_SAMPLES: &[ExplicitSample] = &[
    ExplicitSample {
        rtype: "A",
        rdata: "192.0.2.1",
        reference_type: "A",
        reference_rdata: "192.0.2.1",
        wire: "c0000201",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "A6",
        rdata: "\\# 17 0020010db8000000000000000000000001",
        reference_type: "A6",
        reference_rdata: "0 2001:db8::1",
        wire: "0020010db8000000000000000000000001",
        bind: Support::Required,
        ldns: Support::Skip,
    },
    ExplicitSample {
        rtype: "AAAA",
        rdata: "2001:db8::1",
        reference_type: "AAAA",
        reference_rdata: "2001:db8::1",
        wire: "20010db8000000000000000000000001",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "AFSDB",
        rdata: "\\# 19 000103616673076578616d706c6503636f6d00",
        reference_type: "AFSDB",
        reference_rdata: "1 afs.example.com.",
        wire: "000103616673076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "AMTRELAY",
        rdata: "\\# 6 0a01cb00710f",
        reference_type: "AMTRELAY",
        reference_rdata: "10 0 1 203.0.113.15",
        wire: "0a01cb00710f",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "ANAME",
        rdata: "target.example.com.",
        reference_type: "TYPE65305",
        reference_rdata: "\\# 20 06746172676574076578616d706c6503636f6d00",
        wire: "06746172676574076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Skip,
    },
    ExplicitSample {
        rtype: "APL",
        rdata: "\\# 7 00011803c00002",
        reference_type: "APL",
        reference_rdata: "1:192.0.2.0/24",
        wire: "00011803c00002",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "ATMA",
        rdata: "\\# 23 0039246f00000000000000000000000000000000000000",
        reference_type: "ATMA",
        reference_rdata: "39.246f.000000000000000000000000000000000000.00",
        wire: "0039246f00000000000000000000000000000000000000",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "AVC",
        rdata: "\\# 6 0568656c6c6f",
        reference_type: "AVC",
        reference_rdata: "\"hello\"",
        wire: "0568656c6c6f",
        bind: Support::Required,
        ldns: Support::Skip,
    },
    ExplicitSample {
        rtype: "BRID",
        rdata: "\\# 2 0001",
        reference_type: "BRID",
        reference_rdata: "\\# 2 0001",
        wire: "0001",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "CAA",
        rdata: "0 issue \"letsencrypt.org\"",
        reference_type: "CAA",
        reference_rdata: "0 issue \"letsencrypt.org\"",
        wire: "000569737375656c657473656e63727970742e6f7267",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "CDNSKEY",
        rdata: "257 3 13 YWJj",
        reference_type: "CDNSKEY",
        reference_rdata: "257 3 13 YWJj",
        wire: "0101030d616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "CDS",
        rdata: "12345 13 2 4949494949494949494949494949494949494949494949494949494949494949",
        reference_type: "CDS",
        reference_rdata: "12345 13 2 4949494949494949494949494949494949494949494949494949494949494949",
        wire: "30390d024949494949494949494949494949494949494949494949494949494949494949",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "CERT",
        rdata: "1 12345 13 YWJj",
        reference_type: "CERT",
        reference_rdata: "1 12345 13 YWJj",
        wire: "000130390d616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "CLA",
        rdata: "\\# 1 01",
        reference_type: "CLA",
        reference_rdata: "\\# 1 01",
        wire: "01",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "CNAME",
        rdata: "target.example.com.",
        reference_type: "CNAME",
        reference_rdata: "target.example.com.",
        wire: "06746172676574076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "CSYNC",
        rdata: "\\# 12 000000010003000440000008",
        reference_type: "CSYNC",
        reference_rdata: "1 3 A AAAA",
        wire: "000000010003000440000008",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DELEG",
        rdata: "\\# 1 00",
        reference_type: "DELEG",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DELEGPARAM",
        rdata: "\\# 1 00",
        reference_type: "DELEGPARAM",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DHCID",
        rdata: "\\# 3 616263",
        reference_type: "DHCID",
        reference_rdata: "YWJj",
        wire: "616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DLV",
        rdata: "\\# 36 30390d024949494949494949494949494949494949494949494949494949494949494949",
        reference_type: "DLV",
        reference_rdata: "12345 13 2 4949494949494949494949494949494949494949494949494949494949494949",
        wire: "30390d024949494949494949494949494949494949494949494949494949494949494949",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DNAME",
        rdata: "target.example.com.",
        reference_type: "DNAME",
        reference_rdata: "target.example.com.",
        wire: "06746172676574076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DNSKEY",
        rdata: "257 3 13 YWJj",
        reference_type: "DNSKEY",
        reference_rdata: "257 3 13 YWJj",
        wire: "0101030d616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DOA",
        rdata: "\\# 1 00",
        reference_type: "DOA",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DS",
        rdata: "12345 13 2 4949494949494949494949494949494949494949494949494949494949494949",
        reference_type: "DS",
        reference_rdata: "12345 13 2 4949494949494949494949494949494949494949494949494949494949494949",
        wire: "30390d024949494949494949494949494949494949494949494949494949494949494949",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "DSYNC",
        rdata: "\\# 30 003b0114ef0b6364732d7363616e6e6572076578616d706c65036e657400",
        reference_type: "DSYNC",
        reference_rdata: "CDS NOTIFY 5359 cds-scanner.example.net.",
        wire: "003b0114ef0b6364732d7363616e6e6572076578616d706c65036e657400",
        bind: Support::Required,
        ldns: Support::Skip,
    },
    ExplicitSample {
        rtype: "EID",
        rdata: "\\# 1 00",
        reference_type: "EID",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "EUI48",
        rdata: "00-11-22-33-44-55",
        reference_type: "EUI48",
        reference_rdata: "00-11-22-33-44-55",
        wire: "001122334455",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "EUI64",
        rdata: "00-11-22-33-44-55-66-77",
        reference_type: "EUI64",
        reference_rdata: "00-11-22-33-44-55-66-77",
        wire: "0011223344556677",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "GID",
        rdata: "\\# 4 000003e8",
        reference_type: "TYPE102",
        reference_rdata: "\\# 4 000003e8",
        wire: "000003e8",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "GPOS",
        rdata: "\\# 23 082d33322e36383832083131362e383635320431302e30",
        reference_type: "GPOS",
        reference_rdata: "\"-32.6882\" \"116.8652\" \"10.0\"",
        wire: "082d33322e36383832083131362e383635320431302e30",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "HHIT",
        rdata: "\\# 1 00",
        reference_type: "HHIT",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "HINFO",
        rdata: "\"x86_64\" \"Linux\"",
        reference_type: "HINFO",
        reference_rdata: "\"x86_64\" \"Linux\"",
        wire: "067838365f3634054c696e7578",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "HIP",
        rdata: "\\# 23 10020003200100107b1a74df365639cc39f1d578616263",
        reference_type: "HIP",
        reference_rdata: "2 200100107B1A74DF365639CC39F1D578 YWJj",
        wire: "10020003200100107b1a74df365639cc39f1d578616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "HTTPS",
        rdata: "1 . alpn=h2",
        reference_type: "HTTPS",
        reference_rdata: "1 . alpn=h2",
        wire: "00010000010003026832",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "IPN",
        rdata: "\\# 1 00",
        reference_type: "IPN",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "IPSECKEY",
        rdata: "\\# 10 0a0102c0000201616263",
        reference_type: "IPSECKEY",
        reference_rdata: "10 1 2 192.0.2.1 YWJj",
        wire: "0a0102c0000201616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "ISDN",
        rdata: "\\# 20 0f31353038363230323830303332313703303034",
        reference_type: "ISDN",
        reference_rdata: "\"150862028003217\" \"004\"",
        wire: "0f31353038363230323830303332313703303034",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "ISO",
        rdata: "\\# 1 00",
        reference_type: "ISO",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "KEY",
        rdata: "257 3 13 YWJj",
        reference_type: "KEY",
        reference_rdata: "257 3 13 YWJj",
        wire: "0101030d616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "KX",
        rdata: "10 mail.example.com.",
        reference_type: "KX",
        reference_rdata: "10 mail.example.com.",
        wire: "000a046d61696c076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "L32",
        rdata: "\\# 6 000ac0000201",
        reference_type: "L32",
        reference_rdata: "10 192.0.2.1",
        wire: "000ac0000201",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "L64",
        rdata: "\\# 10 000a20010db811401000",
        reference_type: "L64",
        reference_rdata: "10 2001:db8:1140:1000",
        wire: "000a20010db811401000",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "LOC",
        rdata: "\\# 16 00121613881b802065bd486000989a68",
        reference_type: "LOC",
        reference_rdata: "37 47 0 N 122 23 0 W 10.00m",
        wire: "00121613881b802065bd486000989a68",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "LP",
        rdata: "\\# 22 000a06746172676574076578616d706c6503636f6d00",
        reference_type: "LP",
        reference_rdata: "10 target.example.com.",
        wire: "000a06746172676574076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "MB",
        rdata: "\\# 18 046d61696c076578616d706c6503636f6d00",
        reference_type: "MB",
        reference_rdata: "mail.example.com.",
        wire: "046d61696c076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "MD",
        rdata: "\\# 18 046d61696c076578616d706c6503636f6d00",
        reference_type: "MD",
        reference_rdata: "mail.example.com.",
        wire: "046d61696c076578616d706c6503636f6d00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "MF",
        rdata: "\\# 18 046d61696c076578616d706c6503636f6d00",
        reference_type: "MF",
        reference_rdata: "mail.example.com.",
        wire: "046d61696c076578616d706c6503636f6d00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "MG",
        rdata: "\\# 18 046d61696c076578616d706c6503636f6d00",
        reference_type: "MG",
        reference_rdata: "mail.example.com.",
        wire: "046d61696c076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "MINFO",
        rdata: "\\# 38 05726d61696c076578616d706c6503636f6d0005656d61696c076578616d706c6503636f6d00",
        reference_type: "MINFO",
        reference_rdata: "rmail.example.com. email.example.com.",
        wire: "05726d61696c076578616d706c6503636f6d0005656d61696c076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "MR",
        rdata: "\\# 18 046d61696c076578616d706c6503636f6d00",
        reference_type: "MR",
        reference_rdata: "mail.example.com.",
        wire: "046d61696c076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "MX",
        rdata: "10 mail.example.com.",
        reference_type: "MX",
        reference_rdata: "10 mail.example.com.",
        wire: "000a046d61696c076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NAPTR",
        rdata: "100 10 \"S\" \"SIP+D2U\" \"\" _sip._udp.example.com.",
        reference_type: "NAPTR",
        reference_rdata: "100 10 \"S\" \"SIP+D2U\" \"\" _sip._udp.example.com.",
        wire: "0064000a0153075349502b44325500045f736970045f756470076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NID",
        rdata: "\\# 10 000a00144fffff20ee64",
        reference_type: "NID",
        reference_rdata: "10 0014:4fff:ff20:ee64",
        wire: "000a00144fffff20ee64",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NIMLOC",
        rdata: "\\# 1 00",
        reference_type: "NIMLOC",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NINFO",
        rdata: "\\# 5 04696e666f",
        reference_type: "NINFO",
        reference_rdata: "\"info\"",
        wire: "04696e666f",
        bind: Support::Required,
        ldns: Support::Skip,
    },
    ExplicitSample {
        rtype: "NS",
        rdata: "ns.example.com.",
        reference_type: "NS",
        reference_rdata: "ns.example.com.",
        wire: "026e73076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NSAP",
        rdata: "\\# 20 47000580005a0000000001e133ffffff00016100",
        reference_type: "NSAP",
        reference_rdata: "0x47000580005a0000000001e133ffffff00016100",
        wire: "47000580005a0000000001e133ffffff00016100",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NSAP-PTR",
        rdata: "\\# 20 06746172676574076578616d706c6503636f6d00",
        reference_type: "NSAP-PTR",
        reference_rdata: "target.example.com.",
        wire: "06746172676574076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NSEC",
        rdata: "\\# 24 046e657874076578616d706c6503636f6d00000440000008",
        reference_type: "NSEC",
        reference_rdata: "next.example.com. A AAAA",
        wire: "046e657874076578616d706c6503636f6d00000440000008",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NSEC3",
        rdata: "\\# 33 0100000c04aabbccdd1417f3df17b2b2adaef615257de4d2020b80ac6c7c000140",
        reference_type: "NSEC3",
        reference_rdata: "1 0 12 aabbccdd 2vptu5timamqttgl4luu9kg21e0aor3s A",
        wire: "0100000c04aabbccdd1417f3df17b2b2adaef615257de4d2020b80ac6c7c000140",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NSEC3PARAM",
        rdata: "\\# 9 0100000c04aabbccdd",
        reference_type: "NSEC3PARAM",
        reference_rdata: "1 0 12 aabbccdd",
        wire: "0100000c04aabbccdd",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NULL",
        rdata: "\\# 4 01020304",
        reference_type: "NULL",
        reference_rdata: "\\# 4 01020304",
        wire: "01020304",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NXNAME",
        rdata: "\\# 0 ",
        reference_type: "NXNAME",
        reference_rdata: "\\# 0",
        wire: "",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "NXT",
        rdata: "\\# 22 046e657874076578616d706c6503636f6d0040000008",
        reference_type: "NXT",
        reference_rdata: "next.example.com. A AAAA",
        wire: "046e657874076578616d706c6503636f6d0040000008",
        bind: Support::Required,
        ldns: Support::Skip,
    },
    ExplicitSample {
        rtype: "OPENPGPKEY",
        rdata: "\\# 3 616263",
        reference_type: "OPENPGPKEY",
        reference_rdata: "YWJj",
        wire: "616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "PTR",
        rdata: "target.example.com.",
        reference_type: "PTR",
        reference_rdata: "target.example.com.",
        wire: "06746172676574076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "PX",
        rdata: "\\# 43 000a066d6170383232076578616d706c6503636f6d00076d617078343030076578616d706c6503636f6d00",
        reference_type: "PX",
        reference_rdata: "10 map822.example.com. mapx400.example.com.",
        wire: "000a066d6170383232076578616d706c6503636f6d00076d617078343030076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "RESINFO",
        rdata: "\\# 16 0f716e616d656d696e3d657874657272",
        reference_type: "RESINFO",
        reference_rdata: "\"qnamemin=exterr\"",
        wire: "0f716e616d656d696e3d657874657272",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "RKEY",
        rdata: "257 3 13 YWJj",
        reference_type: "TYPE57",
        reference_rdata: "\\# 7 0101030d616263",
        wire: "0101030d616263",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "RP",
        rdata: "hostmaster.example.com. info.example.com.",
        reference_type: "RP",
        reference_rdata: "hostmaster.example.com. info.example.com.",
        wire: "0a686f73746d6173746572076578616d706c6503636f6d0004696e666f076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "RRSIG",
        rdata: "A 13 2 300 20260101000000 20250101000000 12345 example.com. YWJj",
        reference_type: "RRSIG",
        reference_rdata: "A 13 2 300 20260101000000 20250101000000 12345 example.com. YWJj",
        wire: "00010d020000012c6955b900677485803039076578616d706c6503636f6d00616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "RT",
        rdata: "\\# 22 000a06746172676574076578616d706c6503636f6d00",
        reference_type: "RT",
        reference_rdata: "10 target.example.com.",
        wire: "000a06746172676574076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "SIG",
        rdata: "\\# 34 00010d020000012c6955b900677485803039076578616d706c6503636f6d00616263",
        reference_type: "SIG",
        reference_rdata: "A 13 2 300 20260101000000 20250101000000 12345 example.com. YWJj",
        wire: "00010d020000012c6955b900677485803039076578616d706c6503636f6d00616263",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "SINK",
        rdata: "\\# 1 00",
        reference_type: "SINK",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "SMIMEA",
        rdata: "3 1 1 abcd",
        reference_type: "SMIMEA",
        reference_rdata: "3 1 1 abcd",
        wire: "030101abcd",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "SOA",
        rdata: "ns.example.com. hostmaster.example.com. 1 3600 600 604800 300",
        reference_type: "SOA",
        reference_rdata: "ns.example.com. hostmaster.example.com. 1 3600 600 604800 300",
        wire: "026e73076578616d706c6503636f6d000a686f73746d6173746572076578616d706c6503636f6d000000000100000e100000025800093a800000012c",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "SPF",
        rdata: "\"v=spf1 -all\"",
        reference_type: "SPF",
        reference_rdata: "\"v=spf1 -all\"",
        wire: "0b763d73706631202d616c6c",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "SRV",
        rdata: "10 5 5060 sip.example.com.",
        reference_type: "SRV",
        reference_rdata: "10 5 5060 sip.example.com.",
        wire: "000a000513c403736970076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "SSHFP",
        rdata: "1 1 abababababababababababababababababababab",
        reference_type: "SSHFP",
        reference_rdata: "1 1 abababababababababababababababababababab",
        wire: "0101abababababababababababababababababababab",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "SVCB",
        rdata: "1 svc.example.com. alpn=h2 port=443",
        reference_type: "SVCB",
        reference_rdata: "1 svc.example.com. alpn=h2 port=443",
        wire: "000103737663076578616d706c6503636f6d00000100030268320003000201bb",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "TA",
        rdata: "\\# 36 30390d024949494949494949494949494949494949494949494949494949494949494949",
        reference_type: "TA",
        reference_rdata: "12345 13 2 4949494949494949494949494949494949494949494949494949494949494949",
        wire: "30390d024949494949494949494949494949494949494949494949494949494949494949",
        bind: Support::Required,
        ldns: Support::Skip,
    },
    ExplicitSample {
        rtype: "TALINK",
        rdata: "\\# 36 0470726576076578616d706c6503636f6d00046e657874076578616d706c6503636f6d00",
        reference_type: "TALINK",
        reference_rdata: "prev.example.com. next.example.com.",
        wire: "0470726576076578616d706c6503636f6d00046e657874076578616d706c6503636f6d00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "TLSA",
        rdata: "3 1 1 abcd",
        reference_type: "TLSA",
        reference_rdata: "3 1 1 abcd",
        wire: "030101abcd",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "TXT",
        rdata: "\"hello\"",
        reference_type: "TXT",
        reference_rdata: "\"hello\"",
        wire: "0568656c6c6f",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "UID",
        rdata: "\\# 4 000003e8",
        reference_type: "TYPE101",
        reference_rdata: "\\# 4 000003e8",
        wire: "000003e8",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "UINFO",
        rdata: "\\# 4 696e666f",
        reference_type: "TYPE100",
        reference_rdata: "\\# 4 696e666f",
        wire: "696e666f",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "UNECE",
        rdata: "\\# 1 00",
        reference_type: "UNECE",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Skip,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "UNSPEC",
        rdata: "\\# 1 00",
        reference_type: "UNSPEC",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "URI",
        rdata: "10 1 \"https://example.com/\"",
        reference_type: "URI",
        reference_rdata: "10 1 \"https://example.com/\"",
        wire: "000a000168747470733a2f2f6578616d706c652e636f6d2f",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "WALLET",
        rdata: "\\# 1 00",
        reference_type: "WALLET",
        reference_rdata: "\\# 1 00",
        wire: "00",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "WKS",
        rdata: "\\# 16 c0000201060000000000000000000080",
        reference_type: "WKS",
        reference_rdata: "192.0.2.1 6 80",
        wire: "c0000201060000000000000000000080",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "X25",
        rdata: "\\# 13 0c333131303631373030393536",
        reference_type: "X25",
        reference_rdata: "\"311061700956\"",
        wire: "0c333131303631373030393536",
        bind: Support::Required,
        ldns: Support::Required,
    },
    ExplicitSample {
        rtype: "ZONEMD",
        rdata: "\\# 54 000000010101abababababababababababababababababababababababababababababababababababababababababababababababab",
        reference_type: "ZONEMD",
        reference_rdata: "1 1 1 abababababababababababababababababababababababababababababababababababababababababababababababab",
        wire: "000000010101abababababababababababababababababababababababababababababababababababababababababababababababab",
        bind: Support::Required,
        ldns: Support::Required,
    },
];

struct IanaType {
    name: String,
    code: u16,
    meta: bool,
}

#[test]
fn registry_matches_iana() {
    let registry: BTreeSet<&str> = RECORD_TYPES.iter().copied().collect();
    assert_eq!(
        registry.len(),
        RECORD_TYPES.len(),
        "RECORD_TYPES contains duplicate names"
    );

    for name in RECORD_TYPES {
        let rtype = RecordType::from_str(name).unwrap();
        assert!(!rtype.is_meta(), "{name} must not be a meta type");
        assert_eq!(
            rtype.to_string(),
            *name,
            "registry name mismatch for {name}"
        );
        assert_eq!(
            RecordType::from_u16(rtype.as_u16()),
            rtype,
            "type code roundtrip failed for {name}"
        );
    }

    let iana = match iana_types() {
        Ok(iana) => iana,
        Err(err) => {
            eprintln!("skipping live IANA registry check: {err}");
            return;
        }
    };

    let data_types: BTreeSet<&str> = iana
        .iter()
        .filter(|record_type| !record_type.meta)
        .map(|record_type| record_type.name.as_str())
        .collect();

    let missing: Vec<&str> = data_types
        .iter()
        .copied()
        .filter(|name| !registry.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "add these IANA record types to RECORD_TYPES: {missing:?}"
    );

    for record_type in iana.iter().filter(|record_type| !record_type.meta) {
        let rtype = RecordType::from_str(&record_type.name).unwrap();
        assert_eq!(
            rtype.as_u16(),
            record_type.code,
            "type code for {} differs from IANA",
            record_type.name
        );
    }

    for record_type in iana.iter().filter(|record_type| record_type.meta) {
        let rtype = RecordType::from_str(&record_type.name)
            .unwrap_or_else(|_| panic!("meta type {} is not in RecordType", record_type.name));
        assert!(rtype.is_meta(), "{} must be a meta type", record_type.name);
        assert!(
            !registry.contains(record_type.name.as_str()),
            "{} must not be stored in a zone",
            record_type.name
        );
        assert!(
            RData::parse(rtype, "\\# 0").is_none(),
            "{} must not accept RDATA",
            record_type.name
        );
    }

    let extras: Vec<&str> = registry.difference(&data_types).copied().collect();
    eprintln!("record types not currently assigned by IANA: {extras:?}");
}

#[test]
fn samples_cover_every_registry_type() {
    let samples = build_samples();
    let sampled: BTreeSet<&str> = samples.iter().map(|sample| sample.rtype.as_str()).collect();
    assert_eq!(
        sampled.len(),
        samples.len(),
        "SAMPLES contains duplicate record types"
    );

    for name in RECORD_TYPES {
        assert!(sampled.contains(name), "missing sample for {name}");
    }

    if let Ok(iana) = iana_types() {
        for record_type in iana.iter().filter(|record_type| !record_type.meta) {
            assert!(
                sampled.contains(record_type.name.as_str()),
                "missing sample for IANA type {}",
                record_type.name
            );
        }
    }

    for sample in &samples {
        let rtype = RecordType::from_u16(sample.code);
        assert_eq!(rtype.as_u16(), sample.code);
        assert!(
            RData::parse(rtype, &sample.rdata).is_some(),
            "sample for {} is not parseable",
            sample.rtype
        );
        assert!(
            decode_hex(&sample.wire).is_some(),
            "invalid expected wire for {}",
            sample.rtype
        );
    }
}

#[test]
fn typed_support_report() {
    let samples = build_samples();
    let mut untyped = Vec::new();
    let mut typed = 0usize;

    for sample in &samples {
        let rtype = RecordType::from_u16(sample.code);
        let data = RData::parse(rtype, &sample.rdata)
            .unwrap_or_else(|| panic!("failed to parse sample for {}", sample.rtype));
        assert_eq!(data.rtype(), rtype, "type mismatch for {}", sample.rtype);
        if matches!(data, RData::Unknown { .. }) {
            untyped.push(sample.rtype.clone());
        } else {
            typed += 1;
        }
    }

    untyped.sort();
    eprintln!("typed support: {typed} typed, {} untyped", untyped.len());
    eprintln!("untyped record types: {}", untyped.join(", "));
}

#[test]
fn unknown_type_codes_roundtrip() {
    let unassigned = [
        0u16, 71, 98, 110, 127, 129, 248, 265, 32767, 61441, 65279, 65280, 65534, 65535,
    ];
    for code in unassigned {
        let rtype = RecordType::Unknown(code);
        assert_eq!(rtype.as_u16(), code);
        assert_eq!(rtype.to_string(), format!("TYPE{code}"));
        assert_eq!(RecordType::from_str(&format!("TYPE{code}")), Ok(rtype));

        let rdata = format!("\\# 2 {:02x}{:02x}", code & 0xff, code >> 8);
        let parsed = RData::parse(rtype, &rdata).unwrap();
        assert_eq!(parsed.rtype(), rtype);
        assert_eq!(parsed.to_wire(), vec![code as u8, (code >> 8) as u8]);
        assert_eq!(
            parsed.to_string(),
            format!("\\# 2 {:02x}{:02x}", code & 0xff, code >> 8)
        );
    }
}

#[test]
fn wire_matches_reference_implementations() {
    let bind = tool_available("named-compilezone");
    let ldns = tool_available("ldns-read-zone");
    if !bind && !ldns {
        eprintln!(
            "skipping reference parity: install named-compilezone or ldns-read-zone to enable it"
        );
        return;
    }

    let samples = build_samples();
    let dir = TempDir::new();
    let mut bind_checked = 0usize;
    let mut ldns_checked = 0usize;
    let mut checked = BTreeSet::new();
    let mut optional_skips = Vec::new();

    for sample in &samples {
        let rtype = RecordType::from_u16(sample.code);
        let data = RData::parse(rtype, &sample.rdata)
            .unwrap_or_else(|| panic!("failed to parse sample for {}", sample.rtype));
        let expected = decode_hex(&sample.wire)
            .unwrap_or_else(|| panic!("invalid expected wire for {}", sample.rtype));
        assert_eq!(
            data.to_wire(),
            expected,
            "wire mismatch for our parser: {}",
            sample.rtype
        );

        if bind && sample.bind != Support::Skip {
            match bind_reference_wire(dir.path(), sample) {
                Ok(wire) => {
                    assert_eq!(wire, expected, "BIND wire mismatch for {}", sample.rtype);
                    bind_checked += 1;
                    checked.insert(sample.rtype.clone());
                }
                Err(err) if sample.bind == Support::Optional => {
                    optional_skips.push(format!("BIND {}: {err}", sample.rtype));
                }
                Err(err) => panic!("{err}"),
            }
        }

        if ldns
            && sample.ldns != Support::Skip
            && !LDNS_PRESENTATION_QUIRKS.contains(&sample.rtype.as_str())
        {
            match ldns_reference_wire(dir.path(), sample) {
                Ok(wire) => {
                    assert_eq!(wire, expected, "ldns wire mismatch for {}", sample.rtype);
                    ldns_checked += 1;
                    checked.insert(sample.rtype.clone());
                }
                Err(err) if sample.ldns == Support::Optional => {
                    optional_skips.push(format!("ldns {}: {err}", sample.rtype));
                }
                Err(err) => panic!("{err}"),
            }
        }
    }

    eprintln!(
        "reference parity: BIND checked {bind_checked}, ldns checked {ldns_checked}, optional skips {}",
        optional_skips.len()
    );
    for skip in &optional_skips {
        eprintln!("  {skip}");
    }

    if bind && ldns {
        let missing: Vec<&str> = samples
            .iter()
            .filter(|sample| sample.bind == Support::Required || sample.ldns == Support::Required)
            .filter(|sample| !checked.contains(&sample.rtype))
            .map(|sample| sample.rtype.as_str())
            .collect();
        assert!(
            missing.is_empty(),
            "no reference implementation checked: {missing:?}"
        );
    }
}

fn build_samples() -> Vec<Sample> {
    let mut samples: Vec<Sample> = EXPLICIT_SAMPLES
        .iter()
        .map(|sample| {
            let rtype = RecordType::from_str(sample.rtype).unwrap();
            Sample {
                rtype: sample.rtype.to_string(),
                code: rtype.as_u16(),
                rdata: sample.rdata.to_string(),
                reference_type: sample.reference_type.to_string(),
                reference_rdata: sample.reference_rdata.to_string(),
                wire: sample.wire.to_string(),
                bind: sample.bind,
                ldns: sample.ldns,
            }
        })
        .collect();

    let mut known: BTreeSet<String> = samples.iter().map(|sample| sample.rtype.clone()).collect();

    for name in RECORD_TYPES {
        if known.insert((*name).to_string()) {
            let code = RecordType::from_str(name).unwrap().as_u16();
            samples.push(default_sample(name, code));
        }
    }

    if let Ok(iana) = iana_types() {
        for record_type in iana.iter().filter(|record_type| !record_type.meta) {
            if known.insert(record_type.name.clone()) {
                samples.push(default_sample(&record_type.name, record_type.code));
            }
        }
    }

    samples.sort_by(|left, right| left.rtype.cmp(&right.rtype));
    samples
}

fn default_sample(name: &str, code: u16) -> Sample {
    Sample {
        rtype: name.to_string(),
        code,
        rdata: "\\# 0".to_string(),
        reference_type: format!("TYPE{code}"),
        reference_rdata: "\\# 0".to_string(),
        wire: String::new(),
        bind: Support::Optional,
        ldns: Support::Optional,
    }
}

fn iana_types() -> &'static Result<Vec<IanaType>, String> {
    static TYPES: OnceLock<Result<Vec<IanaType>, String>> = OnceLock::new();
    TYPES.get_or_init(|| {
        fetch_iana_csv()
            .map(|csv| parse_iana_csv(&csv))
            .map_err(|err| format!("could not fetch the IANA registry: {err}"))
    })
}

fn parse_iana_csv(csv: &str) -> Vec<IanaType> {
    let mut types = Vec::new();
    for line in csv.lines().skip(1) {
        let mut fields = line.split(',');
        let Some(name) = fields
            .next()
            .map(str::trim)
            .map(|name| name.trim_matches('"'))
        else {
            continue;
        };
        let Some(value) = fields.next().map(str::trim) else {
            continue;
        };
        if name.is_empty() || name == "Reserved" || name == "Unassigned" || name == "Private use" {
            continue;
        }
        let Ok(code) = value.parse::<u16>() else {
            continue;
        };
        let name = if name == "*" {
            "ANY".to_string()
        } else {
            name.to_string()
        };
        types.push(IanaType {
            name,
            code,
            meta: META_CODES.contains(&code),
        });
    }
    types
}

fn fetch_iana_csv() -> Result<String, String> {
    let mut errors = Vec::new();
    match fetch_iana_http() {
        Ok(csv) => return Ok(csv),
        Err(err) => errors.push(format!("http: {err}")),
    }
    match fetch_with_command("curl", &["-fsSL", "--max-time", "15", IANA_CSV_URL]) {
        Ok(csv) => return Ok(csv),
        Err(err) => errors.push(format!("curl: {err}")),
    }
    match fetch_with_command("wget", &["-qO-", "--timeout=15", IANA_CSV_URL]) {
        Ok(csv) => return Ok(csv),
        Err(err) => errors.push(format!("wget: {err}")),
    }
    Err(errors.join("; "))
}

fn fetch_iana_http() -> Result<String, String> {
    let address = ("www.iana.org", 80)
        .to_socket_addrs()
        .map_err(|err| format!("resolve failed: {err}"))?
        .next()
        .ok_or_else(|| "no address for www.iana.org".to_string())?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(10))
        .map_err(|err| format!("connect failed: {err}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .map_err(|err| err.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|err| err.to_string())?;
    let request = "GET /assignments/dns-parameters/dns-parameters-4.csv HTTP/1.0\r\nHost: www.iana.org\r\nConnection: close\r\nUser-Agent: hackflare-dns-test\r\n\r\n";
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("write failed: {err}"))?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|err| format!("read failed: {err}"))?;
    let response = String::from_utf8(response).map_err(|err| err.to_string())?;
    let (headers, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| "malformed HTTP response".to_string())?;
    let status = headers.lines().next().unwrap_or_default();
    if !status.contains(" 200 ") {
        return Err(format!("unexpected HTTP status: {status}"));
    }
    if headers.lines().any(|line| {
        let line = line.to_ascii_lowercase();
        line.starts_with("transfer-encoding:") && line.contains("chunked")
    }) {
        return decode_chunked(body);
    }
    Ok(body.to_string())
}

fn fetch_with_command(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run {command}: {err}"))?;
    if !output.status.success() {
        return Err(format!("{command} exited with {}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|err| err.to_string())
}

fn decode_chunked(body: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = body;
    loop {
        let (size_line, after) = rest
            .split_once("\r\n")
            .ok_or_else(|| "malformed chunked body".to_string())?;
        let size = usize::from_str_radix(size_line.trim(), 16)
            .map_err(|err| format!("invalid chunk size: {err}"))?;
        if size == 0 {
            return Ok(out);
        }
        let chunk = after
            .get(..size)
            .ok_or_else(|| "truncated chunk".to_string())?;
        out.push_str(chunk);
        rest = after
            .get(size + 2..)
            .ok_or_else(|| "missing chunk terminator".to_string())?;
    }
}

fn tool_available(name: &str) -> bool {
    Command::new(name)
        .arg("-h")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Render a zone that contains the sample plus the records BIND requires.
fn render_zone(sample: &Sample) -> String {
    const DEFAULT_SOA: &str = "ns.example.com. hostmaster.example.com. 1 3600 600 604800 300";
    let soa = if sample.rtype == "SOA" {
        sample.reference_rdata.as_str()
    } else {
        DEFAULT_SOA
    };
    let case = if sample.rtype == "SOA" {
        String::new()
    } else {
        format!(
            "case.example.com. IN {} {}\n",
            sample.reference_type, sample.reference_rdata
        )
    };
    format!(
        "$TTL 300\nexample.com. IN SOA {soa}\nexample.com. IN NS ns.example.com.\nns.example.com. IN A 192.0.2.53\n{case}"
    )
}

fn bind_reference_wire(dir: &Path, sample: &Sample) -> Result<Vec<u8>, String> {
    let name = safe_file_name(&sample.rtype);
    let zone = dir.join(format!("{name}.bind.zone"));
    let raw = dir.join(format!("{name}.bind.raw"));
    fs::write(&zone, render_zone(sample)).map_err(|err| err.to_string())?;
    let output = Command::new("named-compilezone")
        .arg("-F")
        .arg("raw")
        .arg("-o")
        .arg(&raw)
        .arg("example.com")
        .arg(&zone)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "named-compilezone failed for {}: {}",
            sample.rtype,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let bytes = fs::read(&raw).map_err(|err| err.to_string())?;
    parse_bind_raw(&bytes, sample)
        .ok_or_else(|| format!("record missing from BIND raw output for {}", sample.rtype))
}

fn parse_bind_raw(bytes: &[u8], sample: &Sample) -> Option<Vec<u8>> {
    let owner = if sample.rtype == "SOA" {
        "example.com."
    } else {
        "case.example.com."
    };
    let expected_owner = encode_name_labels_vec(owner);
    let expected_type = sample.code;

    // Raw zone files start with a 24-byte header, then length-prefixed RRsets.
    let mut pos = 24usize;
    while pos + 20 <= bytes.len() {
        let start = pos;
        let total = read_u32(bytes, pos)? as usize;
        pos += 4;
        pos += 2; // class
        let rtype = read_u16(bytes, pos)?;
        pos += 2;
        pos += 2; // covers
        pos += 4; // ttl
        let count = read_u32(bytes, pos)?;
        pos += 4;

        let end = start.checked_add(total)?;
        if end > bytes.len() || end <= start {
            return None;
        }

        for _ in 0..count {
            let name_len = usize::from(read_u16(bytes, pos)?);
            pos += 2;
            let name = bytes.get(pos..pos.checked_add(name_len)?)?;
            pos += name_len;

            let rdlen = usize::from(read_u16(bytes, pos)?);
            pos += 2;
            let rdata = bytes.get(pos..pos.checked_add(rdlen)?)?;
            pos += rdlen;

            if name == expected_owner.as_slice() && rtype == expected_type {
                return Some(rdata.to_vec());
            }
        }
        pos = end;
    }
    None
}

fn ldns_reference_wire(dir: &Path, sample: &Sample) -> Result<Vec<u8>, String> {
    let zone = dir.join(format!("{}.ldns.zone", safe_file_name(&sample.rtype)));
    fs::write(&zone, render_zone(sample)).map_err(|err| err.to_string())?;
    let output = Command::new("ldns-read-zone")
        .arg("-u")
        .arg(&sample.reference_type)
        .arg(&zone)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "ldns-read-zone failed for {}: {}",
            sample.rtype,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let owner = if sample.rtype == "SOA" {
        "example.com."
    } else {
        "case.example.com."
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if !line.starts_with(owner) {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let Some(index) = tokens.iter().position(|token| *token == "\\#") else {
            continue;
        };
        let length = tokens
            .get(index + 1)
            .and_then(|token| token.parse::<usize>().ok())
            .ok_or_else(|| format!("invalid RFC 3597 length for {}", sample.rtype))?;
        let wire = decode_hex(&tokens[index + 2..].concat())
            .ok_or_else(|| format!("invalid RFC 3597 hex for {}", sample.rtype))?;
        if wire.len() != length {
            return Err(format!(
                "RFC 3597 length mismatch for {}: expected {length}, got {}",
                sample.rtype,
                wire.len()
            ));
        }
        return Ok(wire);
    }
    Err(format!(
        "record missing from ldns output for {}",
        sample.rtype
    ))
}

fn safe_file_name(name: &str) -> String {
    name.replace('-', "_")
}

fn read_u16(bytes: &[u8], pos: usize) -> Option<u16> {
    let bytes: [u8; 2] = bytes.get(pos..pos + 2)?.try_into().ok()?;
    Some(u16::from_be_bytes(bytes))
}

fn read_u32(bytes: &[u8], pos: usize) -> Option<u32> {
    let bytes: [u8; 4] = bytes.get(pos..pos + 4)?.try_into().ok()?;
    Some(u32::from_be_bytes(bytes))
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = hex_digit(bytes[index])?;
        let low = hex_digit(bytes[index + 1])?;
        out.push((high << 4) | low);
        index += 2;
    }
    Some(out)
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "hackflare-dns-parity-{}-{counter}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
