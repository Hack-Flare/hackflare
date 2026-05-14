# Integrating hackflare_dns

This guide shows how to integrate the `hackflare_dns` crate

## Table of Contents

- [Basic Setup](#basic-setup)
- [PostgreSQL Persistence](#postgresql-persistence)
- [Zone Management](#zone-management)
- [Server Startup](#server-startup)
- [Configuration](#configuration)

## Basic Setup

### 1. Add the Crate to Your `Cargo.toml`

```toml
[dependencies]
hackflare_dns = { path = "./hackflare_dns" }
tokio = { version = "1", features = ["full"] }
```

### 2. Create a Simple In-Memory Nameserver

```rust
use hackflare_dns::{Nameserver, NsConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NsConfig {
        bind_addr: "127.0.0.1".to_string(),
        port: 5353,
        zone_file: None,
        database_url: None,
    };

    let nameserver = Nameserver::new(config);

    // Create a zone
    nameserver.create_zone("example.com");

    // Add some records
    nameserver.add_record("example.com", "@", "A", 3600, "192.0.2.1");
    nameserver.add_record("example.com", "www", "A", 3600, "192.0.2.2");
    nameserver.add_record("example.com", "mail", "MX", 3600, "10 mail.example.com");

    println!("Zones: {:?}", nameserver.list_zones());

    // Start the DNS server
    nameserver.run()?;

    Ok(())
}
```

## PostgreSQL Persistence

### 1. Set Up PostgreSQL

Create a PostgreSQL database for DNS zones:

```bash
createdb dns_server
```

### 2. Create Nameserver with Persistence

```rust
use hackflare_dns::{Nameserver, NsConfig, DnsConfig};
use hackflare_dns::ns::PostgresPersistence;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create persistence backend
    let persistence = Arc::new(PostgresPersistence::new(
        "postgresql://localhost/dns_server"
    ));

    // Initialize the database schema
    persistence.init_schema()?;

    // Create nameserver with persistence
    let config = NsConfig {
        bind_addr: "0.0.0.0".to_string(),
        port: 53,
        zone_file: None,
        database_url: Some("postgresql://localhost/dns_server".to_string()),
    };

    let nameserver = Nameserver::with_persistence(
        config,
        DnsConfig::from_env(),
        persistence.clone(),
    );

    // Load persisted zones on startup
    nameserver.authority.load_zones_from_storage().await?;

    println!("Loaded zones: {:?}", nameserver.list_zones());

    // Start the server in a separate thread
    std::thread::spawn(move || {
        nameserver.run().expect("Failed to run server");
    });

    // Add a new zone and persist it
    nameserver.create_zone("mysite.com");
    nameserver.add_record("mysite.com", "www", "A", 300, "10.0.0.1");

    // Save the zone to database
    nameserver.authority.save_zone_to_storage("mysite.com").await?;

    // Keep the server running
    std::thread::sleep(std::time::Duration::from_secs(u64::MAX));

    Ok(())
}
```

## Zone Management

### Creating Zones

```rust
nameserver.create_zone("example.com");
nameserver.create_zone("test.org");
```

### Adding Records

The crate supports all standard DNS record types:

```rust
// A record (IPv4)
nameserver.add_record("example.com", "www", "A", 300, "192.0.2.1");

// AAAA record (IPv6)
nameserver.add_record("example.com", "www", "AAAA", 300, "2001:db8::1");

// CNAME record
nameserver.add_record("example.com", "alias", "CNAME", 300, "www.example.com");

// MX record
nameserver.add_record("example.com", "@", "MX", 3600, "10 mail.example.com");

// TXT record
nameserver.add_record("example.com", "@", "TXT", 3600, "v=spf1 ~all");

// NS record
nameserver.add_record("example.com", "@", "NS", 3600, "ns1.example.com");

// SOA record (auto-created per zone, but you can override)
nameserver.add_record("example.com", "@", "SOA", 3600, 
    "ns1.example.com hostmaster.example.com 2024010101 3600 1800 604800 86400");
```

### Listing Zones

```rust
let zones = nameserver.list_zones();
for zone in zones {
    println!("Zone: {}", zone);
}
```

### Removing Records

```rust
nameserver.remove_record("example.com", "www", "A");
nameserver.remove_record("example.com", "www", "AAAA");
```

### Deleting Zones

```rust
let deleted = nameserver.delete_zone("example.com");
if deleted {
    println!("Zone deleted");
} else {
    println!("Zone not found");
}
```

## Server Startup

### Running the Server

The server runs indefinitely once started:

```rust
match nameserver.run() {
    Ok(_) => println!("Server exited"),
    Err(e) => eprintln!("Server error: {}", e),
}
```

### Running in Background

To run the server while handling other tasks:

```rust
use std::thread;

let nameserver_clone = Arc::new(nameserver);

// Spawn server in background thread
let server_handle = thread::spawn({
    let ns = Arc::clone(&nameserver_clone);
    move || ns.run()
});

// Do other work...

// Wait for server to finish (if ever)
let _ = server_handle.join();
```

## Configuration

### Environment Variables

Configuration can be loaded from environment variables:

```bash
# DNS configuration
export SOA_MNAME=ns1.example.com
export SOA_RNAME=admin@example.com
export SOA_SERIAL=2024010101
export SOA_REFRESH=3600
export SOA_RETRY=1800
export SOA_EXPIRE=604800
export SOA_MINIMUM=86400
export SOA_TTL=3600

# Nameserver configuration
export BIND_ADDR=0.0.0.0
export BIND_PORT=53
export DATABASE_URL=postgresql://user:pass@localhost/dns_db
```

### Programmatic Configuration

```rust
use hackflare_dns::{DnsConfig, NsConfig};

let dns_config = DnsConfig {
    recursion_enabled: true,
    soa_mname: "ns1.example.com".to_string(),
    soa_rname: "admin@example.com".to_string(),
    soa_serial: "2024010101".to_string(),
    soa_refresh: "3600".to_string(),
    soa_retry: "1800".to_string(),
    soa_expire: "604800".to_string(),
    soa_minimum: "86400".to_string(),
    soa_ttl: "3600".to_string(),
    udp_size: 512,
    udp_attempts: 4,
    udp_timeout: std::time::Duration::from_millis(2500),
    recursion_rounds: 8,
    recursion_debug: false,
    root_hints_file: None,
    database_url: None,
};

let ns_config = NsConfig {
    bind_addr: "0.0.0.0".to_string(),
    port: 53,
    zone_file: None,
    database_url: Some("postgresql://localhost/dns".to_string()),
};

let nameserver = Nameserver::with_dns_config(ns_config, dns_config);
```

## Custom Persistence Backends

To implement your own persistence backend (e.g., for MySQL, DynamoDB, Redis):

```rust
use hackflare_dns::ns::{ZonePersistence, PersistedZone, PersistedRecord};
use async_trait::async_trait;
use std::error::Error;

struct MyCustomBackend {
    // Your fields here
}

#[async_trait]
impl ZonePersistence for MyCustomBackend {
    async fn load_zones(&self) -> Result<Vec<PersistedZone>, Box<dyn Error>> {
        // Implementation
        todo!()
    }

    async fn load_zone(&self, zone_name: &str) -> Result<Option<PersistedZone>, Box<dyn Error>> {
        // Implementation
        todo!()
    }

    async fn save_zone(&self, zone: &PersistedZone) -> Result<(), Box<dyn Error>> {
        // Implementation
        todo!()
    }

    async fn delete_zone(&self, zone_name: &str) -> Result<(), Box<dyn Error>> {
        // Implementation
        todo!()
    }

    async fn save_record(&self, zone_name: &str, record: &PersistedRecord) 
        -> Result<(), Box<dyn Error>> {
        // Implementation
        todo!()
    }

    async fn delete_record(&self, zone_name: &str, name: &str, rtype: &str) 
        -> Result<(), Box<dyn Error>> {
        // Implementation
        todo!()
    }
}

// Use your custom backend
let my_backend = Arc::new(MyCustomBackend { /* ... */ });
let nameserver = Nameserver::with_persistence(config, dns_config, my_backend);
```

## Testing Locally

### Query with `dig`

```bash
# Query a local zone
dig @127.0.0.1 -p 5353 www.example.com

# Query the SOA
dig @127.0.0.1 -p 5353 example.com SOA

# Trace a CNAME
dig @127.0.0.1 -p 5353 alias.example.com
```

### DNS Load Testing

```bash
# Use dnsbench or similar tools
# https://github.com/miekg/dns

# Simple test with bash
for i in {1..100}; do 
    dig @127.0.0.1 -p 5353 www.example.com +short &
done
wait
```

## Troubleshooting

### PostgreSQL Connection Errors

Check that PostgreSQL is running and accessible:

```bash
psql postgresql://localhost/dns_server -c "SELECT 1"
```

### Zone Loading Issues

Ensure zones were actually saved before trying to load them:

```rust
// Debug: print zones after loading
let zones = nameserver.list_zones();
println!("Loaded {} zones", zones.len());
for zone in zones {
    println!("  - {}", zone);
}
```
