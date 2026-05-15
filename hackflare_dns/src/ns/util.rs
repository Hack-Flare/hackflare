use std::sync::Arc;

use hickory_server::net::runtime::TokioRuntimeProvider;
use hickory_server::store::in_memory::InMemoryZoneHandler;
use hickory_server::zone_handler::ZoneHandler;

/// Erase `Arc<InMemoryZoneHandler>` to `Arc<dyn ZoneHandler>`.
///
/// This is a concrete non-generic function so rust-analyzer can resolve
/// the return-position unsizing coercion directly.
pub(crate) fn erase_to_dyn(
    handler: Arc<InMemoryZoneHandler<TokioRuntimeProvider>>,
) -> Arc<dyn ZoneHandler> {
    handler
}
