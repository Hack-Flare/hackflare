use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

const ROOT_SERVERS: [&str; 13] = [
    "198.41.0.4",
    "170.247.170.2",
    "192.33.4.12",
    "199.7.91.13",
    "192.203.230.10",
    "192.5.5.241",
    "192.112.36.4",
    "198.97.190.53",
    "192.36.148.17",
    "192.58.128.30",
    "193.0.14.129",
    "199.7.83.42",
    "202.12.27.33",
];

fn parse_root_hints(content: &str) -> Vec<String> {
    let mut ips = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        for token in trimmed.split_whitespace() {
            if let Ok(ip) = token.parse::<Ipv4Addr>() {
                ips.push(ip.to_string());
            }
        }
    }
    ips.sort();
    ips.dedup();
    ips
}

fn root_hints_content() -> String {
    let mut out = String::from("; Auto-generated root hints by Hackflare\n");
    for ip in ROOT_SERVERS {
        out.push_str(ip);
        out.push('\n');
    }
    out
}

fn root_hint_candidate_paths() -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    if let Ok(path) = env::var("HACKFLARE_ROOT_HINTS_FILE")
        && !path.trim().is_empty()
    {
        paths.push(path);
    }
    paths.push("/etc/hackflare/root.hints".to_string());
    paths.push("/etc/bind/db.root".to_string());
    paths.push("/etc/named.root".to_string());
    paths.push("./root.hints".to_string());
    paths.push("/tmp/hackflare/root.hints".to_string());
    paths
}

fn try_create_root_hints_file(path: &str) -> bool {
    let p = Path::new(path);
    if p.exists() {
        return false;
    }
    if let Some(parent) = p.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    fs::write(p, root_hints_content()).is_ok()
}

fn load_root_hint_servers_internal(custom_path: Option<&PathBuf>) -> Vec<String> {
    if let Some(path) = custom_path
        && let Ok(content) = fs::read_to_string(path)
    {
        let parsed = parse_root_hints(&content);
        if !parsed.is_empty() {
            return parsed;
        }
    }

    let paths = root_hint_candidate_paths();

    for path in &paths {
        if let Ok(content) = fs::read_to_string(path) {
            let parsed = parse_root_hints(&content);
            if !parsed.is_empty() {
                return parsed;
            }
        }
    }

    for path in &paths {
        if try_create_root_hints_file(path)
            && let Ok(content) = fs::read_to_string(path)
        {
            let parsed = parse_root_hints(&content);
            if !parsed.is_empty() {
                return parsed;
            }
        }
    }

    ROOT_SERVERS.iter().map(|&s| s.to_string()).collect()
}

pub(super) struct RootHints {
    servers: Vec<String>,
}

impl RootHints {
    pub(super) fn load() -> Self {
        Self {
            servers: load_root_hint_servers_internal(None),
        }
    }

    pub(super) fn servers(&self) -> &[String] {
        &self.servers
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn root_hint_parser_deduplicates_ips() {
        let hints = parse_root_hints("; comment\n198.41.0.4 198.41.0.4\n170.247.170.2\n");
        assert_eq!(
            hints,
            vec!["170.247.170.2".to_string(), "198.41.0.4".to_string()]
        );
    }
}
