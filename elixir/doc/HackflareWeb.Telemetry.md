# `HackflareWeb.Telemetry`

# `child_spec`

Returns a specification to start this module under a supervisor.

See `Supervisor`.

# `metrics`

Returns the list of metrics to be monitored by the telemetry system.

Includes metrics for:
- Phoenix HTTP endpoints and routing
- Phoenix LiveView channels
- Erlang VM memory usage and scheduler metrics

These metrics can be exported to monitoring services or displayed in
the LiveDashboard during development.

## Returns

  List of `Telemetry.Metrics` metric definitions

# `start_link`

Starts the Telemetry supervisor.

This function initializes the telemetry supervision tree, which manages
periodic measurements and metric collection for the application.

## Parameters

  - `arg` - Arguments passed from the supervisor

## Returns

  - `{:ok, pid}` - PID of the telemetry supervisor
  - `{:error, reason}` - If startup fails

---

*Consult [api-reference.md](api-reference.md) for complete listing*
