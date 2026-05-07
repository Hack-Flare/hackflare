use crate::dns::DnsEngine;
use crate::ns::NsConfig;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use postgres::{Client, NoTls};
use serde_json::json;

pub struct Nameserver {
    pub config: NsConfig,
    pub engine: Option<Arc<DnsEngine>>,
}

static UDP_COUNT: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
static TCP_COUNT: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
static UDP_INFLIGHT: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
static TCP_INFLIGHT: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

const MAX_UDP_INFLIGHT: usize = 1024;
const MAX_TCP_INFLIGHT: usize = 512;

fn s_log(level: &str, message: &str, peer: Option<SocketAddr>) {
    let ts = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => 0,
    };
    let mut obj = serde_json::Map::new();
    obj.insert("ts".to_string(), json!(ts));
    obj.insert("level".to_string(), json!(level));
    obj.insert("msg".to_string(), json!(message));
    if let Some(p) = peer {
        obj.insert("peer".to_string(), json!(p.to_string()));
    }
    if let Ok(s) = serde_json::to_string(&obj) {
        eprintln!("{}", s);
    }
}

impl Nameserver {
    pub fn new(config: NsConfig) -> Self {
        Self {
            config,
            engine: None,
        }
    }

    pub fn with_engine(config: NsConfig, engine: DnsEngine) -> Self {
        Self {
            config,
            engine: Some(Arc::new(engine)),
        }
    }

    fn handle_tcp_connection(
        mut stream: TcpStream,
        engine: Option<Arc<DnsEngine>>,
        peer: SocketAddr,
    ) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

        loop {
            let mut len_buf = [0u8; 2];
            if let Err(e) = stream.read_exact(&mut len_buf) {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => {
                        // client closed connection or reset it; treat as normal
                        return;
                    }
                    _ => {
                        s_log("error", &format!("TCP read length failed: {}", e), Some(peer));
                        return;
                    }
                }
            }
            let len = u16::from_be_bytes(len_buf) as usize;
            let mut msg = vec![0u8; len];
            if let Err(e) = stream.read_exact(&mut msg) {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => {
                        return;
                    }
                    _ => {
                        s_log("error", &format!("TCP read msg failed: {}", e), Some(peer));
                        return;
                    }
                }
            }

            TCP_COUNT.fetch_add(1, Ordering::Relaxed);

            if let Some(engine) = &engine {
                match engine.handle_query(&msg) {
                    Some(resp) => {
                        let rlen = (resp.len() as u16).to_be_bytes();
                        if let Err(e) = stream.write_all(&rlen) {
                            s_log("error", &format!("TCP write len failed: {}", e), Some(peer));
                            return;
                        }
                        if let Err(e) = stream.write_all(&resp) {
                            s_log("error", &format!("TCP write resp failed: {}", e), Some(peer));
                            return;
                        }
                    }
                    None => {
                        if let Err(e) = stream.write_all(&0u16.to_be_bytes()) {
                            match e.kind() {
                                io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset => return,
                                _ => {
                                    s_log("error", &format!("TCP write empty failed: {}", e), Some(peer));
                                    return;
                                }
                            }
                        }
                    }
                }
            } else {
                s_log("warn", "No engine configured; closing TCP connection", Some(peer));
                return;
            }
        }
    }

    pub fn run(&self) -> io::Result<()> {
        let bind_addr = self.config.bind_addr.clone();
        let port = self.config.port;

        let udp_bind = format!("{}:{}", bind_addr, port);
        let udp_socket = UdpSocket::bind(&udp_bind)?;
        udp_socket.set_nonblocking(false)?;

        let tcp_bind = udp_bind.clone();
        let tcp_listener = TcpListener::bind(&tcp_bind)?;
        tcp_listener.set_nonblocking(false)?;

        let engine_arc = self.engine.clone();

        let udp_engine = engine_arc.clone();
        let udp_socket = Arc::new(udp_socket);
        let udp_handle = {
            let sock = udp_socket.clone();
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match sock.recv_from(&mut buf) {
                        Ok((amt, src)) => {
                            let req = buf[..amt].to_vec();
                            if UDP_INFLIGHT.fetch_add(1, Ordering::AcqRel) >= MAX_UDP_INFLIGHT {
                                UDP_INFLIGHT.fetch_sub(1, Ordering::AcqRel);
                                continue;
                            }

                            let send_sock = sock.clone();
                            let engine = udp_engine.clone();

                            if thread::Builder::new()
                                .spawn(move || {
                                UDP_COUNT.fetch_add(1, Ordering::Relaxed);
                                if let Some(engine) = &engine
                                    && let Some(resp) = engine.handle_query(&req)
                                {
                                    let _ = send_sock.send_to(&resp, src);
                                }
                                UDP_INFLIGHT.fetch_sub(1, Ordering::AcqRel);
                            })
                            .is_err()
                            {
                                UDP_INFLIGHT.fetch_sub(1, Ordering::AcqRel);
                            }
                        }
                        Err(e) => {
                            s_log("error", &format!("UDP recv error: {}", e), None);
                            thread::sleep(Duration::from_millis(5));
                        }
                    }
                }
            })
        };

        let tcp_engine = engine_arc.clone();
        let tcp_handle = thread::spawn(move || -> io::Result<()> {
            for stream in tcp_listener.incoming() {
                match stream {
                    Ok(s) => {
                        if TCP_INFLIGHT.fetch_add(1, Ordering::AcqRel) >= MAX_TCP_INFLIGHT {
                            TCP_INFLIGHT.fetch_sub(1, Ordering::AcqRel);
                            continue;
                        }
                        let peer = s
                            .peer_addr()
                            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                        let eng = tcp_engine.clone();
                        if thread::Builder::new()
                            .spawn(move || {
                            Nameserver::handle_tcp_connection(s, eng, peer);
                            TCP_INFLIGHT.fetch_sub(1, Ordering::AcqRel);
                        })
                        .is_err()
                        {
                            TCP_INFLIGHT.fetch_sub(1, Ordering::AcqRel);
                        }
                    }
                    Err(e) => {
                        s_log("error", &format!("TCP accept error: {}", e), None);
                    }
                }
            }
            Ok(())
        });

        // Background flusher: periodically persist accumulated counters to DB
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            let dbu = db_url.clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(5));
                    let u = UDP_COUNT.swap(0, Ordering::Relaxed);
                    let t = TCP_COUNT.swap(0, Ordering::Relaxed);
                    if u == 0 && t == 0 {
                        continue;
                    }
                    if let Ok(mut client) = Client::connect(&dbu, NoTls) {
                        let q = format!(
                            "INSERT INTO dns_query_metrics (id, udp_count, tcp_count, inserted_at, updated_at) VALUES (1, {u}, {t}, now(), now()) ON CONFLICT (id) DO UPDATE SET udp_count = dns_query_metrics.udp_count + {u}, tcp_count = dns_query_metrics.tcp_count + {t}, updated_at = now()"
                        );
                        let _ = client.execute(&q, &[]);
                    } else {
                        s_log("warn", "Failed to connect to DB for metrics flush", None);
                    }
                }
            });
        }

        let _ = udp_handle.join();
        let _ = tcp_handle.join();
        Ok(())
    }
}
