use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

type CacheKey = (String, u16);
type CacheValue = (Vec<u8>, Instant);
type RootCacheValue = (Vec<String>, Vec<String>, Instant);
type DelegationCacheValue = (Vec<String>, Instant);

const MAX_QUERY_CACHE_ENTRIES: usize = 10_000;
const MAX_ROOT_CACHE_ENTRIES: usize = 256;
const MAX_DELEGATION_CACHE_ENTRIES: usize = 1024;
pub(super) const ROOT_CACHE_TTL_SECS: u64 = 86400;

pub(super) struct RecursiveCache {
    query: RwLock<HashMap<CacheKey, CacheValue>>,
    root: RwLock<HashMap<String, RootCacheValue>>,
    delegation: RwLock<HashMap<String, DelegationCacheValue>>,
}

impl RecursiveCache {
    pub(super) fn new() -> Self {
        Self {
            query: RwLock::new(HashMap::new()),
            root: RwLock::new(HashMap::new()),
            delegation: RwLock::new(HashMap::new()),
        }
    }

    fn prune<K, V>(map: &mut HashMap<K, V>, max: usize, expired: impl Fn(&V) -> bool)
    where
        K: Clone + Eq + std::hash::Hash,
    {
        map.retain(|_, v| !expired(v));
        while map.len() > max {
            if let Some(key) = map.keys().next().cloned() {
                map.remove(&key);
            } else {
                break;
            }
        }
    }

    pub(super) fn seed_root_cache(&self, hints: &[String], ttl_secs: u64) {
        let mut roots = self.root.write();
        Self::prune(&mut roots, MAX_ROOT_CACHE_ENTRIES, |v: &RootCacheValue| {
            Instant::now() >= v.2
        });
        if !roots.contains_key("__root__") && !hints.is_empty() {
            let exp = Instant::now() + Duration::from_secs(ttl_secs);
            roots.insert("__root__".to_string(), (Vec::new(), hints.to_vec(), exp));
        }
    }

    pub(super) fn get_query(&self, name: &str, qtype: u16) -> Option<Vec<u8>> {
        let mut q = self.query.write();
        Self::prune(&mut q, MAX_QUERY_CACHE_ENTRIES, |v: &CacheValue| {
            Instant::now() >= v.1
        });
        let key = (name.to_string(), qtype);
        if let Some((data, exp)) = q.get(&key)
            && Instant::now() < *exp
        {
            Some(data.clone())
        } else {
            None
        }
    }

    pub(super) fn put_query(&self, name: &str, qtype: u16, data: Vec<u8>, ttl_secs: u32) {
        let mut q = self.query.write();
        Self::prune(&mut q, MAX_QUERY_CACHE_ENTRIES, |v: &CacheValue| {
            Instant::now() >= v.1
        });
        let exp = Instant::now() + Duration::from_secs(u64::from(ttl_secs));
        q.insert((name.to_string(), qtype), (data, exp));
    }

    pub(super) fn get_delegation(&self, tld: &str) -> Option<Vec<String>> {
        let mut d = self.delegation.write();
        Self::prune(
            &mut d,
            MAX_DELEGATION_CACHE_ENTRIES,
            |v: &DelegationCacheValue| Instant::now() >= v.1,
        );
        if let Some((servers, exp)) = d.get(tld)
            && Instant::now() < *exp
            && !servers.is_empty()
        {
            Some(servers.clone())
        } else {
            None
        }
    }

    pub(super) fn put_delegation(&self, tld: &str, servers: &[String], ttl_secs: u64) {
        let mut d = self.delegation.write();
        Self::prune(
            &mut d,
            MAX_DELEGATION_CACHE_ENTRIES,
            |v: &DelegationCacheValue| Instant::now() >= v.1,
        );
        let exp = Instant::now() + Duration::from_secs(ttl_secs);
        d.insert(tld.to_string(), (servers.to_vec(), exp));
    }

    pub(super) fn get_root_glue(&self) -> Option<Vec<String>> {
        let roots = self.root.read();
        if let Some((_ns, glue, exp)) = roots.get("__root__")
            && Instant::now() < *exp
            && !glue.is_empty()
        {
            Some(glue.clone())
        } else {
            None
        }
    }

    pub(super) fn update_root_cache(
        &self,
        ns_names: &[String],
        glue_ips: &[String],
        ttl_secs: u64,
    ) {
        let mut roots = self.root.write();
        let exp = Instant::now() + Duration::from_secs(ttl_secs);
        roots.insert(
            "__root__".to_string(),
            (ns_names.to_vec(), glue_ips.to_vec(), exp),
        );
    }
}

impl Default for RecursiveCache {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) static CACHE: std::sync::LazyLock<RecursiveCache> =
    std::sync::LazyLock::new(RecursiveCache::new);
