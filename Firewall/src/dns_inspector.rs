use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsDecision {
    Allow,
    Block,
    LogOnly,
}

#[derive(Debug, Clone)]
pub struct DnsInspector {
    blocked_domains: HashSet<String>,
    allowed_tlds: Vec<String>,
    sinkhole_ip: String,
    enabled: bool,
    log_queries: bool,
}

impl DnsInspector {
    pub fn new(
        blocked_domains: Vec<String>,
        allowed_tlds: Vec<String>,
        sinkhole_ip: String,
        enabled: bool,
        log_queries: bool,
    ) -> Self {
        Self {
            blocked_domains: blocked_domains
                .into_iter()
                .map(|value| normalize_domain(&value))
                .collect(),
            allowed_tlds: allowed_tlds.into_iter().map(normalize_domain).collect(),
            sinkhole_ip,
            enabled,
            log_queries,
        }
    }

    pub fn inspect(&self, domain: &str) -> DnsDecision {
        if !self.enabled {
            return DnsDecision::Allow;
        }

        let clean = normalize_domain(domain);

        if self.blocked_domains.contains(&clean) {
            return DnsDecision::Block;
        }

        if self.allowed_tlds.is_empty() {
            return DnsDecision::Allow;
        }

        let is_allowed = self.allowed_tlds.iter().any(|suffix| {
            let suffix = normalize_domain(suffix);
            clean.ends_with(&suffix) || clean == suffix
        });

        if is_allowed {
            DnsDecision::Allow
        } else {
            DnsDecision::LogOnly
        }
    }

    pub fn sinkhole_ip(&self) -> &str {
        &self.sinkhole_ip
    }

    pub fn should_log(&self) -> bool {
        self.log_queries
    }
}

fn normalize_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_bad_domain() {
        let inspector = DnsInspector::new(
            vec!["malware.example".to_string()],
            vec![".example.com".to_string()],
            "127.0.0.1".to_string(),
            true,
            true,
        );
        assert_eq!(inspector.inspect("malware.example"), DnsDecision::Block);
    }

    #[test]
    fn allows_safe_domain() {
        let inspector = DnsInspector::new(
            vec![],
            vec![".hackflare.io".to_string()],
            "127.0.0.1".to_string(),
            true,
            true,
        );
        assert_eq!(inspector.inspect("api.hackflare.io"), DnsDecision::Allow);
    }
}
