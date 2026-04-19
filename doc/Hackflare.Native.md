# `Hackflare.Native`

Rust NIF (Native Implemented Function) bindings for DNS operations.

This module provides Elixir bindings to the high-performance Rust DNS engine.
All DNS operations that are performance-critical are implemented in Rust and
exposed through these NIF stubs. The actual implementation is in `native/core/src/`.

## Manager Functions

- `manager_new/0` - Create a new DNS manager instance
- `manager_create_zone/2` - Create a new DNS zone
- `manager_delete_zone/2` - Delete a DNS zone
- `manager_list_zones/1` - List all zones in the manager

## Record Operations

- `manager_add_record/6` - Add a DNS record to a zone
- `manager_remove_record/4` - Remove a record from a zone
- `manager_find_records/2` or `manager_find_records/3` - Query records by name and optional type

## Query Handling

- `engine_handle_query/2` - Process a raw DNS query and return response

## Nameserver Control

- `manager_start_nameserver/3` - Start the nameserver listening on bind:port

## Error Handling

If the native library is not loaded (e.g., in development without compilation),
all functions raise `:nif_not_loaded` error. This is normal behavior and allows
the application to start in development environments.

## Performance

These functions are designed to be fast. DNS query handling in particular uses
Rust's zero-copy capabilities for minimal memory allocation and maximum throughput.

# `engine_handle_query`

Handles a raw DNS query and returns the response.

Processes an incoming DNS query packet and returns the appropriate response.
This is the core DNS query handling function used by the nameserver.

## Parameters

  - `mgr` - DNS manager reference
  - `query` - Raw DNS query packet (typically binary data from the network)

## Returns

  Raw DNS response packet (binary)

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

# `manager_add_record`

Adds a DNS record to a zone.

Creates a resource record in the specified zone. Multiple records with the same
name but different types (or data) can coexist.

## Parameters

  - `mgr` - DNS manager reference
  - `zone` - Zone name where the record will be added
  - `name` - Record name (fully qualified domain name)
  - `type` - DNS record type as atom (`:A`, `:AAAA`, `:CNAME`, `:MX`, `:TXT`, etc.)
  - `ttl` - Time to live in seconds (non-negative integer)
  - `data` - Record data (format depends on type, typically a string or list)

## Returns

  `:ok` - Record added successfully
  `{:error, reason}` - If addition fails (e.g., invalid type, zone not found)

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

# `manager_create_zone`

Creates a new DNS zone in the manager.

A zone represents a domain namespace (e.g., "example.com"). Records can be
added to the zone after creation.

## Parameters

  - `mgr` - DNS manager reference from `manager_new/0`
  - `name` - Zone name as a string (e.g., "example.com")

## Returns

  `:ok` - Zone created successfully
  `{:error, reason}` - If zone creation fails (e.g., zone already exists)

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

# `manager_delete_zone`

Deletes a DNS zone from the manager.

Removes the zone and all its associated records. This operation cannot be undone.

## Parameters

  - `mgr` - DNS manager reference
  - `name` - Zone name to delete

## Returns

  `:ok` - Zone deleted successfully
  `{:error, reason}` - If deletion fails (e.g., zone not found)

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

# `manager_find_records`

Finds DNS records matching a name and optional type.

Queries the manager for records. If a type is not specified, all record types
matching the name are returned.

## Parameters

  - `mgr` - DNS manager reference
  - `name` - Record name to search for
  - `type` - Optional: Specific record type to filter by (default: `nil` returns all types)

## Returns

  List of matching records with their details
  Empty list if no records found

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

# `manager_list_zones`

Lists all zones currently in the DNS manager.

## Parameters

  - `mgr` - DNS manager reference

## Returns

  List of zone names (strings)

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

# `manager_new`

Creates a new DNS manager instance.

The manager is responsible for storing zones and handling DNS record operations.
This should typically be called once during application startup.

## Returns

  Reference to the created DNS manager (opaque term)

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

# `manager_remove_record`

Removes a DNS record from a zone.

Deletes all records matching the specified name and type from the zone.

## Parameters

  - `mgr` - DNS manager reference
  - `zone` - Zone name containing the record
  - `name` - Record name to remove
  - `type` - DNS record type (`:A`, `:AAAA`, `:CNAME`, etc.)

## Returns

  `:ok` - Record removed successfully
  `{:error, reason}` - If removal fails (e.g., record not found)

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

# `manager_start_nameserver`

Starts the DNS nameserver listening on the specified address and port.

Begins listening for incoming DNS queries on UDP. This is a blocking operation
that runs in the background on the Rust side.

## Parameters

  - `mgr` - DNS manager reference
  - `bind` - IP address to bind to (e.g., "0.0.0.0", "127.0.0.1")
  - `port` - UDP port to listen on (typically 53 for DNS)

## Returns

  `:ok` - Nameserver started successfully
  `{:error, reason}` - If startup fails (e.g., port already in use)

## Errors

  `:nif_not_loaded` - If the native library is not compiled/loaded

---

*Consult [api-reference.md](api-reference.md) for complete listing*
