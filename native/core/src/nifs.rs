use crate::dns::engine::DnsEngine;
use crate::dns::DnsManager;
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
    Ok(guard.add_record(&zone_name, &name, &rtype, ttl, &data))
}

#[rustler::nif]
fn manager_remove_record(
    resource: ResourceArc<DnsManagerResource>,
    zone_name: String,
    name: String,
    rtype: String,
) -> NifResult<bool> {
    let mut guard = resource.0.lock().unwrap();
    Ok(guard.remove_record(&zone_name, &name, &rtype))
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

#[rustler::nif]
fn manager_save_to_file(
    resource: ResourceArc<DnsManagerResource>,
    path: String,
) -> NifResult<bool> {
    let guard = resource.0.lock().unwrap();
    match guard.save_to_file(&path) {
        Ok(_) => Ok(true),
        Err(e) => {
            eprintln!("Failed to save DNS manager to {}: {}", path, e);
            Ok(false)
        }
    }
}

#[rustler::nif]
fn manager_load_from_file(path: String) -> NifResult<Option<ResourceArc<DnsManagerResource>>> {
    match crate::dns::DnsManager::load_from_file(&path) {
        Ok(mgr) => Ok(Some(ResourceArc::new(DnsManagerResource(Mutex::new(mgr))))),
        Err(e) => {
            eprintln!("Failed to load DNS manager from {}: {}", path, e);
            Ok(None)
        }
    }
}

#[allow(non_local_definitions)]
pub fn init(env: Env, _info: Term) -> bool {
    let _ = rustler::resource!(DnsManagerResource, env);
    true
}
