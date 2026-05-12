use crate::dns::engine::DnsEngine;
use crate::ns::authority::AuthorityStore;
use crate::ns::NsConfig;
use hickory_server::net::xfer::Protocol;
use hickory_server::proto::op::{DnsResponse, Message, Metadata, ResponseCode};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::zone_handler::MessageResponseBuilder;
use once_cell::sync::Lazy;
use postgres::{Client, NoTls};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use tokio::runtime::Runtime;

static UDP_COUNT: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
static TCP_COUNT: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));

fn s_log(level: &str, message: &str, peer: Option<SocketAddr>) {
    let ts = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => 0,
    };
    let mut obj = serde_json::Map::new();
    obj.insert("ts".to_string(), serde_json::json!(ts));
    obj.insert("level".to_string(), serde_json::json!(level));
    obj.insert("msg".to_string(), serde_json::json!(message));
    if let Some(p) = peer {
        obj.insert("peer".to_string(), serde_json::json!(p.to_string()));
    }
    if let Ok(s) = serde_json::to_string(&obj) {
        eprintln!("{}", s);
    }
}

fn record_request(protocol: Protocol) {
    match protocol {
        Protocol::Udp => {
            UDP_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        Protocol::Tcp => {
            TCP_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
}

fn spawn_metrics_flusher(db_url: String) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(5));
        let udp = UDP_COUNT.swap(0, Ordering::Relaxed);
        let tcp = TCP_COUNT.swap(0, Ordering::Relaxed);
        if udp == 0 && tcp == 0 {
            continue;
        }
        if let Ok(mut client) = Client::connect(&db_url, NoTls) {
            let query = format!(
                "INSERT INTO dns_query_metrics (id, udp_count, tcp_count, inserted_at, updated_at) VALUES (1, {udp}, {tcp}, now(), now()) ON CONFLICT (id) DO UPDATE SET udp_count = dns_query_metrics.udp_count + {udp}, tcp_count = dns_query_metrics.tcp_count + {tcp}, updated_at = now()"
            );
            let _ = client.execute(&query, &[]);
        } else {
            s_log("warn", "Failed to connect to DB for metrics flush", None);
        }
    });
}

pub(crate) struct HickoryRequestHandler {
    authority: Arc<AuthorityStore>,
    engine: Arc<DnsEngine>,
}

impl HickoryRequestHandler {
    pub(crate) fn new(authority: Arc<AuthorityStore>, engine: Arc<DnsEngine>) -> Self {
        Self { authority, engine }
    }
}

#[async_trait::async_trait]
impl RequestHandler for HickoryRequestHandler {
    async fn handle_request<R: ResponseHandler, T: hickory_server::net::runtime::Time>(
        &self,
        request: &Request,
        mut response_handle: R,
    ) -> ResponseInfo {
        record_request(request.protocol());

        if self
            .authority
            .contains_zone_for(request.request_info().unwrap().query.name())
            .await
        {
            return self
                .authority
                .handle_request::<R, T>(request, response_handle)
                .await;
        }

        let response_bytes = self.engine.handle_query(request.as_slice());
        let response = match response_bytes {
            Some(bytes) => match Message::from_vec(bytes.as_slice()) {
                Ok(message) => match DnsResponse::from_message(message) {
                    Ok(response) => Some(response),
                    Err(err) => {
                        s_log("warn", &format!("failed to decode DNS response: {}", err), Some(request.src()));
                        None
                    }
                },
                Err(err) => {
                    s_log("warn", &format!("failed to parse DNS response bytes: {}", err), Some(request.src()));
                    None
                }
            },
            None => None,
        };

        if let Some(response) = response {
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
                    s_log("error", &format!("error responding to request: {}", err), Some(request.src()));
                    let mut metadata = Metadata::response_from_request(&request.metadata);
                    metadata.response_code = ResponseCode::ServFail;
                    let fallback = MessageResponseBuilder::from_message_request(request).build_no_records(metadata);
                    response_handle.send_response(fallback).await.unwrap_or_else(|_| {
                        ResponseInfo::from(hickory_server::proto::op::Header {
                            metadata,
                            counts: hickory_server::proto::op::HeaderCounts::default(),
                        })
                    })
                }
            }
        } else {
            let mut metadata = Metadata::response_from_request(&request.metadata);
            metadata.response_code = ResponseCode::ServFail;
            let message_response = MessageResponseBuilder::from_message_request(request).build_no_records(metadata);

            match response_handle.send_response(message_response).await {
                Ok(info) => info,
                Err(err) => {
                    s_log("error", &format!("error responding to request: {}", err), Some(request.src()));
                    ResponseInfo::from(hickory_server::proto::op::Header {
                        metadata,
                        counts: hickory_server::proto::op::HeaderCounts::default(),
                    })
                }
            }
        }
    }
}

pub(crate) fn run_with_hickory(
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