# `Hackflare.Nameserver`

Manages the DNS nameserver process and native Rust integration.

This GenServer acts as a bridge between the Elixir application and the Rust-based
DNS nameserver implementation. It initializes and manages the lifetime of the native
DNS manager, which handles all DNS query processing and zone management.

## Responsibilities

- Create and maintain the native DNS manager instance
- Start the nameserver listening on the configured bind address and port
- Handle GenServer lifecycle (start, stop, restart)
- Provide interface to DNS operations via `Hackflare.Native`

## Configuration

The nameserver is configured via the `:dns` application environment variable,
which should contain:

    config :hackflare,
      dns: [
        bind: "0.0.0.0",
        port: 53
      ]

## Native Integration

This module uses Rustler NIF bindings in `Hackflare.Native` to communicate with
the Rust nameserver implementation. The native manager handles the actual DNS
protocol parsing, zone storage, and query resolution.

# `child_spec`

Returns a specification to start this module under a supervisor.

See `Supervisor`.

# `start_link`

Starts the DNS nameserver GenServer.

This function is called by the supervisor to initialize the nameserver process.
It registers the process under the module name so it can be called globally.

## Parameters

  - `_opts` - Supervisor options (unused)

## Returns

  - `{:ok, pid}` - PID of the started nameserver process
  - `{:error, reason}` - If startup fails

---

*Consult [api-reference.md](api-reference.md) for complete listing*
