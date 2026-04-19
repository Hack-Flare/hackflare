use crate::dns::engine::DnsEngine;
use crate::dns::{DnsManager, Record as DnsRecord};
use crate::ns::{Nameserver, NsConfig};
use rustler::{Env, NifResult, ResourceArc, Term};
use serde_json;
use std::sync::Mutex;

pub struct DnsManagerResource(pub Mutex<DnsManager>);

#[rustler::nif]
fn manager_new() -> NifResult<ResourceArc<DnsManagerResource>> {
    let mgr = DnsManager::new();
    Ok(ResourceArc::new(DnsManagerResource(Mutex::new(mgr))))
}

#[rustler::nif]
fn manager_create_zone(resource: ResourceArc<DnsManagerResource>, name: String) -> NifResult<bool> {
    let mut guard = resource.0.lock().unwrap();
    guard.create_zone(name);
    Ok(true)
}

#[rustler::nif]
fn manager_delete_zone(resource: ResourceArc<DnsManagerResource>, name: String) -> NifResult<bool> {
    let mut guard = resource.0.lock().unwrap();
    Ok(guard.delete_zone(&name))
}

#[rustler::nif]
fn manager_add_record(
    resource: ResourceArc<DnsManagerResource>,
    zone_name: String,
    name: String,
    rtype: String,
    ttl: u32,
    data: String,
) -> NifResult<bool> {
    let mut guard = resource.0.lock().unwrap();
    if let Some(zone) = guard.get_zone_mut(&zone_name) {
        zone.add_record(DnsRecord::new(name, rtype, ttl, data));
        Ok(true)
    } else {
        Ok(false)
    }
}

#[rustler::nif]
fn manager_remove_record(
    resource: ResourceArc<DnsManagerResource>,
    zone_name: String,
    name: String,
    rtype: String,
) -> NifResult<bool> {
    let mut guard = resource.0.lock().unwrap();
    if let Some(zone) = guard.get_zone_mut(&zone_name) {
        Ok(zone.remove_record(&name, &rtype))
    } else {
        Ok(false)
    }
}

#[rustler::nif]
fn manager_list_zones(resource: ResourceArc<DnsManagerResource>) -> NifResult<String> {
    let guard = resource.0.lock().unwrap();
    let list = guard.list_zones();
    let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string());
    Ok(json)
}

#[rustler::nif]
fn manager_find_records(
    resource: ResourceArc<DnsManagerResource>,
    name: String,
    rtype: Option<String>,
) -> NifResult<String> {
    let guard = resource.0.lock().unwrap();
    let recs = guard.find_records(&name, rtype.as_deref());
    let json = serde_json::to_string(&recs).unwrap_or_else(|_| "[]".to_string());
    Ok(json)
}

#[rustler::nif]
fn engine_handle_query(
    resource: ResourceArc<DnsManagerResource>,
    query: Vec<u8>,
) -> NifResult<Option<Vec<u8>>> {
    let guard = resource.0.lock().unwrap();
    let manager_clone = guard.clone();
    drop(guard);
    let engine = DnsEngine::new(manager_clone);
    Ok(engine.handle_query(&query))
}

#[rustler::nif]
fn manager_start_nameserver(
    resource: ResourceArc<DnsManagerResource>,
    bind_addr: String,
    port: u16,
) -> NifResult<bool> {
    let guard = resource.0.lock().unwrap();
    let manager_clone = guard.clone();
    drop(guard);

    std::thread::spawn(move || {
        let engine = DnsEngine::new(manager_clone);
        let config = NsConfig {
            bind_addr,
            port,
            zone_file: None,
        };
        let ns = Nameserver::with_engine(config, engine);
        if let Err(e) = ns.run() {
            eprintln!("Nameserver exited with error: {}", e);
        }
    });

    Ok(true)
}

pub fn init(env: Env, _info: Term) -> bool {
    rustler::resource!(DnsManagerResource, env);
    true
}
