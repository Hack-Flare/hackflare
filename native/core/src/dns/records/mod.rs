mod a;
mod aaaa;
mod any;
mod caa;
mod cname;
mod dnskey;
mod ds;
mod hinfo;
mod https;
mod loc;
mod mx;
mod ns;
mod nsec;
mod nsec3;
mod ptr;
mod rrsig;
mod soa;
mod srv;
mod sshfp;
mod svcb;
mod tlsa;
mod txt;
mod unknown;

use crate::dns::Record;
use crate::dns::registry::Registry;
use once_cell::sync::Lazy;

pub(crate) static REGISTRY: Lazy<Registry> = Lazy::new(|| {
    let mut r = Registry::new();
    a::register(&mut r);
    ns::register(&mut r);
    cname::register(&mut r);
    soa::register(&mut r);
    ptr::register(&mut r);
    hinfo::register(&mut r);
    mx::register(&mut r);
    txt::register(&mut r);
    aaaa::register(&mut r);
    loc::register(&mut r);
    srv::register(&mut r);
    ds::register(&mut r);
    sshfp::register(&mut r);
    rrsig::register(&mut r);
    nsec::register(&mut r);
    dnskey::register(&mut r);
    nsec3::register(&mut r);
    tlsa::register(&mut r);
    svcb::register(&mut r);
    https::register(&mut r);
    any::register(&mut r);
    caa::register(&mut r);
    unknown::register(&mut r);
    r
});

pub fn encode_by_type(typ: &str, record: &Record) -> Option<Vec<u8>> {
    REGISTRY.get(typ).and_then(|enc| enc(record))
}
