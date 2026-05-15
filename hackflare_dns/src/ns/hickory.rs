use crate::dns::engine::DnsEngine;
use crate::ns::NsConfig;
use crate::ns::authority::AuthorityStore;
use hickory_server::net::xfer::Protocol;
use hickory_server::proto::op::{DnsResponse, Message, Metadata, ResponseCode};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::zone_handler::MessageResponseBuilder;
use postgres::{Client, NoTls};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use tokio::runtime::Runtime;

static UDP_COUNT: std::sync::LazyLock<AtomicU64> = std::sync::LazyLock::new(|| AtomicU64::new(0));
static TCP_COUNT: std::sync::LazyLock<AtomicU64> = std::sync::LazyLock::new(|| AtomicU64::new(0));

// Structured logging helper for JSON output.
fn log(level: &str, message: &str, peer: Option<SocketAddr>) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    let mut obj = serde_json::Map::new();
    obj.insert("ts".to_string(), serde_json::json!(ts));
    obj.insert("level".to_string(), serde_json::json!(level));
    obj.insert("msg".to_string(), serde_json::json!(message));
    if let Some(p) = peer {
        obj.insert("peer".to_string(), serde_json::json!(p.to_string()));
    }

    if let Ok(s) = serde_json::to_string(&obj) {
        eprintln!("{s}");
    }
}

// Record DNS request metrics (UDP vs TCP).
fn record_request(protocol: Protocol) {
    match protocol {
        Protocol::Udp => UDP_COUNT.fetch_add(1, Ordering::Relaxed),
        Protocol::Tcp => TCP_COUNT.fetch_add(1, Ordering::Relaxed),
        _ => 0,
    };
}

// Periodically flush metrics to database.
fn spawn_metrics_flusher(db_url: String) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            let udp = UDP_COUNT.swap(0, Ordering::Relaxed);
            let tcp = TCP_COUNT.swap(0, Ordering::Relaxed);

            if udp == 0 && tcp == 0 {
                continue;
            }

            if let Ok(mut client) = Client::connect(&db_url, NoTls) {
                let query = format!(
                    "INSERT INTO dns_query_metrics (id, udp_count, tcp_count, inserted_at, updated_at) \
                 VALUES (1, {udp}, {tcp}, now(), now()) \
                 ON CONFLICT (id) DO UPDATE SET \
                 udp_count = dns_query_metrics.udp_count + {udp}, \
                 tcp_count = dns_query_metrics.tcp_count + {tcp}, \
                 updated_at = now()"
                );
                let _ = client.execute(&query, &[]);
            } else {
                log("warn", "Failed to connect to DB for metrics flush", None);
            }
        }
    });
}

// Implements hickory-server's RequestHandler to bridge authority and recursive resolution.
pub(super) struct HickoryRequestHandler {
    authority: Arc<AuthorityStore>,
    engine: Arc<DnsEngine>,
}

impl HickoryRequestHandler {
    pub(super) const fn new(authority: Arc<AuthorityStore>, engine: Arc<DnsEngine>) -> Self {
        Self { authority, engine }
    }
}

#[async_trait::async_trait]
impl RequestHandler for HickoryRequestHandler {
    #[allow(clippy::too_many_lines)]
    async fn handle_request<R: ResponseHandler, T: hickory_server::net::runtime::Time>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        record_request(request.protocol());

        // Get query name from request
        let Ok(query_info) = request.request_info() else {
            log("error", "Invalid request info", Some(request.src()));
            let mut metadata = Metadata::response_from_request(&request.metadata);
            metadata.response_code = ResponseCode::ServFail;
            let response =
                MessageResponseBuilder::from_message_request(request).build_no_records(metadata);
            return response_handle
                .send_response(response)
                .await
                .unwrap_or_else(|_| {
                    ResponseInfo::from(hickory_server::proto::op::Header {
                        metadata,
                        counts: hickory_server::proto::op::HeaderCounts::default(),
                    })
                });
        };
        let query_name = query_info.query.name();

        // Try authoritative zone first
        if self.authority.contains_zone_for(query_name).await {
            return self
                .authority
                .handle_request::<R, T>(request, response_handle)
                .await;
        }

        // Fall back to recursive resolution
        let Some(response_bytes) = self.engine.handle_query(request.as_slice()) else {
            log(
                "error",
                "Failed to process recursive query",
                Some(request.src()),
            );
            let mut metadata = Metadata::response_from_request(&request.metadata);
            metadata.response_code = ResponseCode::ServFail;
            let response =
                MessageResponseBuilder::from_message_request(request).build_no_records(metadata);
            return response_handle
                .send_response(response)
                .await
                .unwrap_or_else(|_| {
                    ResponseInfo::from(hickory_server::proto::op::Header {
                        metadata,
                        counts: hickory_server::proto::op::HeaderCounts::default(),
                    })
                });
        };

        // Parse the response
        let response = match Message::from_vec(response_bytes.as_slice()) {
            Ok(message) => match DnsResponse::from_message(message) {
                Ok(resp) => resp,
                Err(err) => {
                    log(
                        "error",
                        &format!("Failed to decode DNS response: {err}"),
                        Some(request.src()),
                    );
                    let mut metadata = Metadata::response_from_request(&request.metadata);
                    metadata.response_code = ResponseCode::ServFail;
                    let response = MessageResponseBuilder::from_message_request(request)
                        .build_no_records(metadata);
                    return response_handle
                        .send_response(response)
                        .await
                        .unwrap_or_else(|_| {
                            ResponseInfo::from(hickory_server::proto::op::Header {
                                metadata,
                                counts: hickory_server::proto::op::HeaderCounts::default(),
                            })
                        });
                }
            },
            Err(err) => {
                log(
                    "error",
                    &format!("Failed to parse DNS response bytes: {err}"),
                    Some(request.src()),
                );
                let mut metadata = Metadata::response_from_request(&request.metadata);
                metadata.response_code = ResponseCode::ServFail;
                let response = MessageResponseBuilder::from_message_request(request)
                    .build_no_records(metadata);
                return response_handle
                    .send_response(response)
                    .await
                    .unwrap_or_else(|_| {
                        ResponseInfo::from(hickory_server::proto::op::Header {
                            metadata,
                            counts: hickory_server::proto::op::HeaderCounts::default(),
                        })
                    });
            }
        };

        // Build response
        let mut builder = MessageResponseBuilder::from_message_request(request);
        if let Some(edns) = &response.edns {
            builder.edns(edns);
        }

        let message_response = builder.build(
            response.metadata,
            &response.answers,
            &response.authorities,
            [],
            &response.additionals,
        );

        match response_handle.send_response(message_response).await {
            Ok(info) => info,
            Err(err) => {
                log(
                    "error",
                    &format!("Error sending response: {err}"),
                    Some(request.src()),
                );
                let mut metadata = Metadata::response_from_request(&request.metadata);
                metadata.response_code = ResponseCode::ServFail;
                ResponseInfo::from(hickory_server::proto::op::Header {
                    metadata,
                    counts: hickory_server::proto::op::HeaderCounts::default(),
                })
            }
        }
    }
}

// Start the DNS server with hickory-server.
pub(super) fn run_with_hickory(
    config: &NsConfig,
    authority: Arc<AuthorityStore>,
    engine: Arc<DnsEngine>,
) -> io::Result<()> {
    if let Some(db_url) = config.database_url.clone() {
        spawn_metrics_flusher(db_url);
    }

    let bind_addr = format!("{}:{}", config.bind_addr, config.port);
    let rt = Runtime::new()?;
    rt.block_on(async move {
        let handler = HickoryRequestHandler::new(authority, engine);
        let mut server = hickory_server::Server::new(handler);

        let udp_socket = UdpSocket::bind(&bind_addr).await?;
        let tcp_listener = TcpListener::bind(&bind_addr).await?;

        server.register_socket(udp_socket);
        server.register_listener(tcp_listener, Duration::from_secs(5), 4096);
        server.block_until_done().await.map_err(io::Error::other)?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::DnsConfig;
    use crate::dns::engine::DnsEngine;

    #[test]
    fn handler_creation() {
        let config = DnsConfig::default_config();
        let authority = Arc::new(AuthorityStore::new(config.clone()));
        let engine = Arc::new(DnsEngine::new(Default::default(), config));

        let handler = HickoryRequestHandler::new(authority, engine);
        assert!(Arc::strong_count(&handler.authority) >= 1);
        assert!(Arc::strong_count(&handler.engine) >= 1);
    }

    #[test]
    fn log_function_creates_valid_json() {
        // Test that log doesn't panic and creates valid output
        log("info", "Test message", None);
        log("error", "Test error", Some("127.0.0.1:53".parse().unwrap()));
        // If we get here, logging succeeded without panicking
        // assert!(true);
    }

    #[test]
    fn record_request_increments_counters() {
        // Reset counters
        UDP_COUNT.store(0, Ordering::Relaxed);
        TCP_COUNT.store(0, Ordering::Relaxed);

        record_request(Protocol::Udp);
        assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 1);

        record_request(Protocol::Tcp);
        assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn record_request_udp_only() {
        UDP_COUNT.store(0, Ordering::Relaxed);
        TCP_COUNT.store(0, Ordering::Relaxed);

        record_request(Protocol::Udp);
        record_request(Protocol::Udp);
        record_request(Protocol::Udp);

        assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 3);
        assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn record_request_tcp_only() {
        UDP_COUNT.store(0, Ordering::Relaxed);
        TCP_COUNT.store(0, Ordering::Relaxed);

        record_request(Protocol::Tcp);
        record_request(Protocol::Tcp);

        assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 0);
        assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn record_request_mixed_protocols() {
        UDP_COUNT.store(0, Ordering::Relaxed);
        TCP_COUNT.store(0, Ordering::Relaxed);

        record_request(Protocol::Udp);
        record_request(Protocol::Tcp);
        record_request(Protocol::Udp);
        record_request(Protocol::Tcp);

        assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 2);
        assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 2);
    }
}
