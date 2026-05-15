use crate::dns::config::DnsConfig;
use crate::ns::NsConfig;
use crate::ns::authority::AuthorityStore;
use hickory_server::net::xfer::Protocol;
use hickory_server::proto::op::{DnsResponse, Message, Metadata, ResponseCode};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::zone_handler::MessageResponseBuilder;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use tokio::runtime::Runtime;

static UDP_COUNT: std::sync::LazyLock<AtomicU64> = std::sync::LazyLock::new(|| AtomicU64::new(0));
static TCP_COUNT: std::sync::LazyLock<AtomicU64> = std::sync::LazyLock::new(|| AtomicU64::new(0));

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

fn record_request(protocol: Protocol) {
    match protocol {
        Protocol::Udp => UDP_COUNT.fetch_add(1, Ordering::Relaxed),
        Protocol::Tcp => TCP_COUNT.fetch_add(1, Ordering::Relaxed),
        _ => 0,
    };
}

pub(super) struct HickoryRequestHandler {
    authority: Arc<AuthorityStore>,
    dns_config: DnsConfig,
}

impl HickoryRequestHandler {
    pub(super) fn new(authority: Arc<AuthorityStore>, dns_config: DnsConfig) -> Self {
        Self { authority, dns_config }
    }
}

async fn send_servfail_response<R: ResponseHandler>(
    request: &Request,
    mut response_handle: R,
    error: impl std::fmt::Display,
) -> ResponseInfo {
    log("error", &error.to_string(), Some(request.src()));
    let mut metadata = Metadata::response_from_request(&request.metadata);
    metadata.response_code = ResponseCode::ServFail;
    let response =
        MessageResponseBuilder::from_message_request(request).build_no_records(metadata);
    response_handle
        .send_response(response)
        .await
        .unwrap_or_else(|_| {
            ResponseInfo::from(hickory_server::proto::op::Header {
                metadata,
                counts: hickory_server::proto::op::HeaderCounts::default(),
            })
        })
}

#[async_trait::async_trait]
impl RequestHandler for HickoryRequestHandler {
    async fn handle_request<R: ResponseHandler, T: hickory_server::net::runtime::Time>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        record_request(request.protocol());

        let Ok(query_info) = request.request_info() else {
            return send_servfail_response(request, response_handle, "Invalid request info").await;
        };
        let query_name = query_info.query.name();

        if self.authority.contains_zone_for(query_name).await {
            return self
                .authority
                .handle_request::<R, T>(request, response_handle)
                .await;
        }

        let qname = query_name.to_utf8();
        let qtype = u16::from(query_info.query.query_type());

        let Some(response_bytes) = crate::dns::recursive::resolve(&qname, qtype, &self.dns_config) else {
            return send_servfail_response(
                request,
                response_handle,
                "Failed to process recursive query",
            )
            .await;
        };

        let response = match Message::from_vec(response_bytes.as_slice()) {
            Ok(message) => match DnsResponse::from_message(message) {
                Ok(resp) => resp,
                Err(err) => {
                    return send_servfail_response(
                        request,
                        response_handle,
                        format!("Failed to decode DNS response: {err}"),
                    )
                    .await;
                }
            },
            Err(err) => {
                return send_servfail_response(
                    request,
                    response_handle,
                    format!("Failed to parse DNS response bytes: {err}"),
                )
                .await;
            }
        };

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

pub(super) fn run_with_hickory(
    config: &NsConfig,
    authority: Arc<AuthorityStore>,
    dns_config: DnsConfig,
) -> io::Result<()> {
    let bind_addr = format!("{}:{}", config.bind_addr, config.port);
    let rt = Runtime::new()?;
    rt.block_on(async move {
        let handler = HickoryRequestHandler::new(authority, dns_config);
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
    use crate::dns::config::DnsConfig;

    #[test]
    fn handler_creation() {
        let config = DnsConfig::default_config();
        let authority = Arc::new(AuthorityStore::new(config.clone()));
        let handler = HickoryRequestHandler::new(authority, config);
        assert!(Arc::strong_count(&handler.authority) >= 1);
    }

    #[test]
    fn log_function_creates_valid_json() {
        log("info", "Test message", None);
        log("error", "Test error", Some("127.0.0.1:53".parse().unwrap()));
    }

    #[test]
    fn record_request_increments_counters() {
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
