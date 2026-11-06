use crate::dns::config::DnsConfig;
use crate::dns::packet::{self, Edns, Header, Message, Protocol, Rcode};
use crate::dns::rr::{RecordType, canonical_name};
use crate::ns::NsConfig;
use crate::ns::authority::AuthorityStore;
use sqlx::PgPool;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::runtime::Runtime;

pub static UDP_COUNT: std::sync::LazyLock<AtomicU64> =
    std::sync::LazyLock::new(|| AtomicU64::new(0));
pub static TCP_COUNT: std::sync::LazyLock<AtomicU64> =
    std::sync::LazyLock::new(|| AtomicU64::new(0));

/// Idle timeout for a DNS-over-TCP connection.
const TCP_READ_TIMEOUT: Duration = Duration::from_secs(5);
/// Largest DNS message that can be read or framed on TCP.
const TCP_MAX_MESSAGE_LEN: usize = u16::MAX as usize;

pub fn log(level: &str, message: &str, peer: Option<SocketAddr>) {
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

pub fn record_request(protocol: Protocol) {
    match protocol {
        Protocol::Udp => UDP_COUNT.fetch_add(1, Ordering::Relaxed),
        Protocol::Tcp => TCP_COUNT.fetch_add(1, Ordering::Relaxed),
    };
}

/// Convert a DNS response code to its RFC mnemonic (for example "NOERROR").
pub fn response_code_mnemonic(code: Rcode) -> &'static str {
    code.mnemonic()
}

pub struct DnsHandler {
    pub authority: Arc<AuthorityStore>,
    dns_config: DnsConfig,
    db: Option<PgPool>,
}

impl DnsHandler {
    pub fn new(authority: Arc<AuthorityStore>, dns_config: DnsConfig, db: Option<PgPool>) -> Self {
        Self {
            authority,
            dns_config,
            db,
        }
    }

    async fn log_query(&self, entry: QueryLogEntry) {
        let Some(db) = &self.db else {
            return;
        };
        let _ = sqlx::query(
            r#"
            INSERT INTO dns_query_logs (query_name, query_type, response_code, source_ip, protocol, response_size, processing_us, answers_count, zone_name)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(entry.query_name)
        .bind(entry.query_type)
        .bind(entry.response_code)
        .bind(entry.source_ip)
        .bind(entry.protocol)
        .bind(entry.response_size)
        .bind(entry.processing_us)
        .bind(entry.answers_count)
        .bind(entry.zone_name)
        .execute(db)
        .await;
    }

    async fn log_reply(&self, reply: &HandlerReply, response_size: usize) {
        let elapsed = reply.started.elapsed();
        let processing_us = i32::try_from(elapsed.as_micros()).unwrap_or(i32::MAX);
        let response_size = i32::try_from(response_size).unwrap_or(i32::MAX);
        let answers_count = i32::try_from(reply.answers_count).unwrap_or(i32::MAX);

        self.log_query(QueryLogEntry {
            query_name: reply.query_name.clone(),
            query_type: reply.query_type.clone(),
            response_code: reply.response_code.mnemonic().to_string(),
            source_ip: reply.source_ip.to_string(),
            protocol: reply.protocol.as_str().to_string(),
            response_size,
            processing_us,
            answers_count,
            zone_name: reply.zone_name.clone(),
        })
        .await;
    }

    async fn handle_query(
        &self,
        query: &Message,
        src: SocketAddr,
        protocol: Protocol,
    ) -> HandlerReply {
        record_request(protocol);
        let started = Instant::now();

        let question = query.questions.first();
        let query_name = question.map_or_else(String::new, |question| question.name.clone());
        let query_type = question.map_or_else(
            || "UNKNOWN".to_string(),
            |question| RecordType::from_u16(question.qtype).to_string(),
        );

        let mut reply = HandlerReply {
            reply: Reply::Message(error_response(
                query,
                Rcode::FormErr,
                self.dns_config.recursion_enabled,
                self.dns_config.max_edns_payload_size,
            )),
            query_name,
            query_type,
            response_code: Rcode::FormErr,
            source_ip: src,
            protocol,
            zone_name: String::new(),
            answers_count: 0,
            started,
        };

        if query.questions.len() != 1 {
            return reply;
        }
        let Some(question) = query.questions.first() else {
            return reply;
        };

        if question.qclass != 1 {
            reply.reply = Reply::Message(error_response(
                query,
                Rcode::Refused,
                self.dns_config.recursion_enabled,
                self.dns_config.max_edns_payload_size,
            ));
            reply.response_code = Rcode::Refused;
            return reply;
        }

        if query.header.opcode() != 0 {
            reply.reply = Reply::Message(error_response(
                query,
                Rcode::NotImp,
                self.dns_config.recursion_enabled,
                self.dns_config.max_edns_payload_size,
            ));
            reply.response_code = Rcode::NotImp;
            return reply;
        }

        if canonical_name(&question.name).is_none() {
            return reply;
        }

        if self.authority.contains_zone_for(&question.name).await {
            reply.zone_name = self
                .authority
                .find_zone_name(&question.name)
                .await
                .unwrap_or_default();
            let response = self.authority.answer(query, src, protocol).await;
            reply.response_code = response.header.rcode();
            reply.answers_count = response.answers.len();
            reply.reply = Reply::Message(response);
            return reply;
        }

        if !self.dns_config.recursion_enabled {
            reply.reply = Reply::Message(error_response(
                query,
                Rcode::Refused,
                false,
                self.dns_config.max_edns_payload_size,
            ));
            reply.response_code = Rcode::Refused;
            return reply;
        }

        let qname = question.name.trim_end_matches('.').to_string();
        let qtype = question.qtype;
        let dns_config = self.dns_config.clone();
        let resolve_qname = qname.clone();

        match tokio::task::spawn_blocking(move || {
            crate::dns::recursive::resolve(&resolve_qname, qtype, &dns_config)
        })
        .await
        {
            Ok(Ok(mut bytes)) => match patch_recursive_response(query, &mut bytes) {
                Some(header) => {
                    reply.response_code = header.rcode();
                    reply.answers_count = usize::from(header.ancount);
                    reply.reply = Reply::Raw(bytes);
                }
                None => {
                    reply.reply = Reply::Message(error_response(
                        query,
                        Rcode::ServFail,
                        true,
                        self.dns_config.max_edns_payload_size,
                    ));
                    reply.response_code = Rcode::ServFail;
                }
            },
            Ok(Err(error)) => {
                log(
                    "error",
                    &format!("recursive resolve failed for {qname}: {error}"),
                    Some(src),
                );
                reply.reply = Reply::Message(error_response(
                    query,
                    Rcode::ServFail,
                    true,
                    self.dns_config.max_edns_payload_size,
                ));
                reply.response_code = Rcode::ServFail;
            }
            Err(error) => {
                log(
                    "error",
                    &format!("recursive resolve task failed for {qname}: {error}"),
                    Some(src),
                );
                reply.reply = Reply::Message(error_response(
                    query,
                    Rcode::ServFail,
                    true,
                    self.dns_config.max_edns_payload_size,
                ));
                reply.response_code = Rcode::ServFail;
            }
        }

        reply
    }
}

struct QueryLogEntry {
    query_name: String,
    query_type: String,
    response_code: String,
    source_ip: String,
    protocol: String,
    response_size: i32,
    processing_us: i32,
    answers_count: i32,
    zone_name: String,
}

struct HandlerReply {
    reply: Reply,
    query_name: String,
    query_type: String,
    response_code: Rcode,
    source_ip: SocketAddr,
    protocol: Protocol,
    zone_name: String,
    answers_count: usize,
    started: Instant,
}

enum Reply {
    Message(Message),
    Raw(Vec<u8>),
}

fn error_response(
    query: &Message,
    rcode: Rcode,
    recursion_available: bool,
    server_payload_size: u16,
) -> Message {
    let mut response = Message::response_to(query);
    response.header.set_recursion_available(recursion_available);
    response.header.set_rcode(rcode);
    response.edns = query
        .edns
        .as_ref()
        .map(|edns| Edns::response_to(edns, server_payload_size));
    response
}

/// Patch a recursive resolver response to match the client query.
///
/// The resolver returns a complete wire message.
/// Rewriting its header keeps the compressed RDATA intact instead of re-encoding decoded records.
fn patch_recursive_response(query: &Message, bytes: &mut [u8]) -> Option<Header> {
    let mut header = Header::parse(bytes).ok()?;
    let mut flags = header.flags;
    flags |= packet::flags::QR | packet::flags::RA;
    flags &= !packet::flags::AA;
    flags = (flags & !packet::flags::RD) | (query.header.flags & packet::flags::RD);
    flags =
        (flags & !packet::flags::OPCODE_MASK) | (query.header.flags & packet::flags::OPCODE_MASK);

    header.id = query.header.id;
    header.flags = flags;
    bytes[0..2].copy_from_slice(&header.id.to_be_bytes());
    bytes[2..4].copy_from_slice(&header.flags.to_be_bytes());
    Some(header)
}

fn max_udp_payload(query: &Message, config: &DnsConfig) -> usize {
    let requested = query
        .edns
        .as_ref()
        .map_or(packet::CLASSIC_UDP_PAYLOAD, |edns| {
            usize::from(edns.udp_payload_size).max(packet::CLASSIC_UDP_PAYLOAD)
        });
    requested.min(usize::from(config.max_edns_payload_size).max(packet::CLASSIC_UDP_PAYLOAD))
}

fn encode_reply(
    reply: &Reply,
    query: &Message,
    protocol: Protocol,
    config: &DnsConfig,
) -> Result<Vec<u8>, packet::PacketError> {
    match reply {
        Reply::Message(message) => {
            if protocol == Protocol::Udp {
                message.encode_udp(max_udp_payload(query, config))
            } else {
                message.encode()
            }
        }
        Reply::Raw(bytes) => {
            if protocol == Protocol::Udp && bytes.len() > max_udp_payload(query, config) {
                let rcode = Header::parse(bytes).map_or(Rcode::ServFail, |header| header.rcode());
                let mut message = error_response(query, rcode, true, config.max_edns_payload_size);
                message.header.set_truncated(true);
                message.encode_udp(max_udp_payload(query, config))
            } else {
                Ok(bytes.clone())
            }
        }
    }
}

async fn handle_udp(socket: Arc<UdpSocket>, handler: Arc<DnsHandler>) -> io::Result<()> {
    let mut buffer = vec![0u8; TCP_MAX_MESSAGE_LEN];
    loop {
        let (len, peer) = match socket.recv_from(&mut buffer).await {
            Ok(received) => received,
            Err(error) => {
                log("error", &format!("UDP receive failed: {error}"), None);
                continue;
            }
        };

        let Ok(query) = Message::parse(&buffer[..len]) else {
            continue;
        };
        if query.header.is_response() {
            continue;
        }

        let socket = socket.clone();
        let handler = handler.clone();
        tokio::spawn(async move {
            let reply = handler.handle_query(&query, peer, Protocol::Udp).await;
            match encode_reply(&reply.reply, &query, Protocol::Udp, &handler.dns_config) {
                Ok(bytes) => {
                    if let Err(error) = socket.send_to(&bytes, peer).await {
                        log("error", &format!("UDP send failed: {error}"), Some(peer));
                    }
                    handler.log_reply(&reply, bytes.len()).await;
                }
                Err(error) => {
                    log(
                        "error",
                        &format!("failed to encode UDP response: {error}"),
                        Some(peer),
                    );
                }
            }
        });
    }
}

async fn handle_tcp_connection(
    stream: TcpStream,
    peer: SocketAddr,
    handler: Arc<DnsHandler>,
) -> io::Result<()> {
    let (mut reader, mut writer) = stream.into_split();

    loop {
        let mut length_bytes = [0u8; 2];
        match tokio::time::timeout(TCP_READ_TIMEOUT, reader.read_exact(&mut length_bytes)).await {
            Ok(Ok(_)) => {}
            Ok(Err(error)) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Ok(Err(error)) => return Err(error),
            Err(_) => return Ok(()),
        }

        let len = usize::from(u16::from_be_bytes(length_bytes));
        if len == 0 {
            return Ok(());
        }

        let mut buffer = vec![0u8; len];
        match tokio::time::timeout(TCP_READ_TIMEOUT, reader.read_exact(&mut buffer)).await {
            Ok(Ok(_)) => {}
            Ok(Err(error)) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Ok(Err(error)) => return Err(error),
            Err(_) => return Ok(()),
        }

        let Ok(query) = Message::parse(&buffer) else {
            continue;
        };
        if query.header.is_response() {
            continue;
        }

        let reply = handler.handle_query(&query, peer, Protocol::Tcp).await;
        let Ok(bytes) = encode_reply(&reply.reply, &query, Protocol::Tcp, &handler.dns_config)
        else {
            continue;
        };
        let Ok(len) = u16::try_from(bytes.len()) else {
            continue;
        };

        writer.write_all(&len.to_be_bytes()).await?;
        writer.write_all(&bytes).await?;
        handler.log_reply(&reply, bytes.len()).await;
    }
}

async fn handle_tcp(listener: TcpListener, handler: Arc<DnsHandler>) -> io::Result<()> {
    loop {
        let (stream, peer) = listener.accept().await?;
        let handler = handler.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_tcp_connection(stream, peer, handler).await {
                log(
                    "error",
                    &format!("TCP connection failed: {error}"),
                    Some(peer),
                );
            }
        });
    }
}

pub fn run(
    config: NsConfig,
    authority: Arc<AuthorityStore>,
    dns_config: DnsConfig,
    db: Option<PgPool>,
) -> io::Result<()> {
    let bind_addr = format!("{}:{}", config.bind_addr, config.port);
    let rt = Runtime::new()?;
    rt.block_on(async move {
        let handler = Arc::new(DnsHandler::new(authority, dns_config, db));
        let udp_socket = Arc::new(UdpSocket::bind(&bind_addr).await?);
        let tcp_listener = TcpListener::bind(&bind_addr).await?;

        tokio::try_join!(
            handle_udp(udp_socket, handler.clone()),
            handle_tcp(tcp_listener, handler)
        )?;
        Ok(())
    })
}
