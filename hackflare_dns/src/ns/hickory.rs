use crate::dns::config::DnsConfig;
use crate::ns::NsConfig;
use crate::ns::authority::AuthorityStore;
use hickory_server::net::xfer::Protocol;
use hickory_server::proto::op::{DnsResponse, Message, Metadata, ResponseCode};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::zone_handler::MessageResponseBuilder;
use sqlx::PgPool;
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
    db: Option<PgPool>,
}

impl HickoryRequestHandler {
    pub(super) fn new(
        authority: Arc<AuthorityStore>,
        dns_config: DnsConfig,
        db: Option<PgPool>,
    ) -> Self {
        Self {
            authority,
            dns_config,
            db,
        }
    }

    async fn log_query(
        &self,
        query_name: &str,
        query_type: &str,
        response_code: &str,
        source_ip: &str,
        protocol: &str,
        response_size: i32,
        elapsed: std::time::Duration,
        answers_count: i32,
    ) {
        let Some(db) = &self.db else {
            return;
        };
        let processing_us = elapsed.as_micros().min(u64::MAX as u128) as i32;
        let _ = sqlx::query(
            r#"
            INSERT INTO dns_query_logs (query_name, query_type, response_code, source_ip, protocol, response_size, processing_us, answers_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(query_name)
        .bind(query_type)
        .bind(response_code)
        .bind(source_ip)
        .bind(protocol)
        .bind(response_size)
        .bind(processing_us)
        .bind(answers_count)
        .execute(db)
        .await;
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
    let response = MessageResponseBuilder::from_message_request(request).build_no_records(metadata);
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
        let start = std::time::Instant::now();
        record_request(request.protocol());

        let Ok(query_info) = request.request_info() else {
            return send_servfail_response(request, response_handle, "Invalid request info").await;
        };
        let query_lower = query_info.query.name().clone();
        let query_name = query_lower.to_utf8();
        let query_type_str = query_info.query.query_type().to_string();
        let source_ip = request.src().to_string();
        let protocol_str = if request.protocol() == Protocol::Udp {
            "UDP"
        } else {
            "TCP"
        };

        let response_code;
        let response_size;
        let answers_count;
        let response_info;

        if self.authority.contains_zone_for(&query_lower).await {
            response_info = self
                .authority
                .handle_request::<R, T>(request, response_handle)
                .await;
            response_code = response_info.response_code.to_string();
            response_size = 0;
            answers_count = 0;
        } else {
            let qname = query_name.clone();
            let qtype = u16::from(query_info.query.query_type());

            let response_bytes = match crate::dns::recursive::resolve(&qname, qtype, &self.dns_config) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("recursive resolve failed for {qname}: {e}");
                    response_info = send_servfail_response(
                        request,
                        response_handle,
                        "Failed to process recursive query",
                    )
                    .await;
                    response_code = "SERVFAIL".to_string();
                    response_size = 0;
                    answers_count = 0;
                    let elapsed = start.elapsed();
                    self.log_query(&query_name, &query_type_str, &response_code, &source_ip, protocol_str, response_size, elapsed, answers_count).await;
                    return response_info;
                }
            };

            response_size = response_bytes.len() as i32;
            let mut response = match Message::from_vec(response_bytes.as_slice()) {
                Ok(message) => match DnsResponse::from_message(message) {
                    Ok(resp) => resp,
                    Err(err) => {
                        response_info = send_servfail_response(
                            request,
                            response_handle,
                            format!("Failed to decode DNS response: {err}"),
                        )
                        .await;
                        response_code = "SERVFAIL".to_string();
                        answers_count = 0;
                        let elapsed = start.elapsed();
                        self.log_query(&query_name, &query_type_str, &response_code, &source_ip, protocol_str, response_size, elapsed, answers_count).await;
                        return response_info;
                    }
                },
                Err(err) => {
                    response_info = send_servfail_response(
                        request,
                        response_handle,
                        format!("Failed to parse DNS response bytes: {err}"),
                    )
                    .await;
                    response_code = "SERVFAIL".to_string();
                    answers_count = 0;
                    let elapsed = start.elapsed();
                    self.log_query(&query_name, &query_type_str, &response_code, &source_ip, protocol_str, response_size, elapsed, answers_count).await;
                    return response_info;
                }
            };

            response_code = response.metadata.response_code.to_string();
            answers_count = response.answers.len() as i32;

            // Build response metadata from the request so the ID matches
            let mut response_meta = Metadata::response_from_request(&request.metadata);
            response_meta.response_code = response.metadata.response_code;
            response_meta.recursion_available = true;
            response_meta.recursion_desired = request.metadata.recursion_desired;
            response_meta.authoritative = false;

            // Clamp UDP response size to prevent amplification attacks
            if request.protocol() == Protocol::Udp
                && response_size as usize > self.dns_config.max_edns_payload_size as usize
            {
                response_meta.truncation = true;
                response.answers.clear();
                response.additionals.clear();
            }

            let mut builder = MessageResponseBuilder::from_message_request(request);
            if let Some(edns) = &response.edns {
                builder.edns(edns);
            }

            let message_response = builder.build(
                response_meta,
                &response.answers,
                &response.authorities,
                [],
                &response.additionals,
            );

            match response_handle.send_response(message_response).await {
                Ok(info) => {
                    response_info = info;
                }
                Err(err) => {
                    log(
                        "error",
                        &format!("Error sending response: {err}"),
                        Some(request.src()),
                    );
                    let mut metadata = Metadata::response_from_request(&request.metadata);
                    metadata.response_code = ResponseCode::ServFail;
                    response_info = ResponseInfo::from(hickory_server::proto::op::Header {
                        metadata,
                        counts: hickory_server::proto::op::HeaderCounts::default(),
                    });
                }
            }
        }

        let elapsed = start.elapsed();
        self.log_query(&query_name, &query_type_str, &response_code, &source_ip, protocol_str, response_size, elapsed, answers_count).await;
        response_info
    }
}

pub fn run_with_hickory(
    config: NsConfig,
    authority: Arc<AuthorityStore>,
    dns_config: DnsConfig,
    db: Option<PgPool>,
) -> io::Result<()> {
    let bind_addr = format!("{}:{}", config.bind_addr, config.port);
    let rt = Runtime::new()?;
    rt.block_on(async move {
        let handler = HickoryRequestHandler::new(authority, dns_config, db);
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
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::dns::config::DnsConfig;

    #[test]
    fn handler_creation() {
        let config = DnsConfig::default_config();
        let authority = Arc::new(AuthorityStore::new(config.clone()));
        let handler = HickoryRequestHandler::new(authority, config, None);
        assert!(Arc::strong_count(&handler.authority) >= 1);
    }

    #[test]
    fn log_function_creates_valid_json() {
        log("info", "Test message", None);
        log("error", "Test error", Some("127.0.0.1:53".parse().unwrap()));
    }

    #[test]
    fn record_request_counters() {
        UDP_COUNT.store(0, Ordering::Relaxed);
        TCP_COUNT.store(0, Ordering::Relaxed);

        record_request(Protocol::Udp);
        assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 1);

        record_request(Protocol::Tcp);
        assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 1);

        UDP_COUNT.store(0, Ordering::Relaxed);
        TCP_COUNT.store(0, Ordering::Relaxed);

        record_request(Protocol::Udp);
        record_request(Protocol::Udp);
        record_request(Protocol::Udp);

        assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 3);
        assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 0);

        UDP_COUNT.store(0, Ordering::Relaxed);
        TCP_COUNT.store(0, Ordering::Relaxed);

        record_request(Protocol::Tcp);
        record_request(Protocol::Tcp);

        assert_eq!(UDP_COUNT.load(Ordering::Relaxed), 0);
        assert_eq!(TCP_COUNT.load(Ordering::Relaxed), 2);

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
