use crate::dns::DnsEngine;
use crate::ns::NsConfig;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use postgres::{Client, NoTls};

pub struct Nameserver {
    pub config: NsConfig,
    pub engine: Option<Arc<DnsEngine>>,
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
        loop {
            let mut len_buf = [0u8; 2];
            if let Err(e) = stream.read_exact(&mut len_buf) {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe => {
                        // client closed connection or reset it; treat as normal
                        return;
                    }
                    _ => {
                        eprintln!("TCP read length failed from {}: {}", peer, e);
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
                        eprintln!("TCP read msg failed from {}: {}", peer, e);
                        return;
                    }
                }
            }

            // increment TCP metric asynchronously
            if let Ok(db_url) = std::env::var("DATABASE_URL") {
                let _ = thread::spawn(move || {
                    if let Ok(mut client) = Client::connect(&db_url, NoTls) {
                        let _ = client.execute("INSERT INTO dns_query_metrics (id, udp_count, tcp_count, inserted_at, updated_at) VALUES (1, 0, 1, now(), now()) ON CONFLICT (id) DO UPDATE SET tcp_count = dns_query_metrics.tcp_count + 1, updated_at = now()", &[]);
                    }
                });
            }

            if let Some(engine) = &engine {
                match engine.handle_query(&msg) {
                    Some(resp) => {
                        let rlen = (resp.len() as u16).to_be_bytes();
                        if let Err(e) = stream.write_all(&rlen) {
                            eprintln!("TCP write len failed to {}: {}", peer, e);
                            return;
                        }
                        if let Err(e) = stream.write_all(&resp) {
                            eprintln!("TCP write resp failed to {}: {}", peer, e);
                            return;
                        }
                    }
                    None => {
                        if let Err(e) = stream.write_all(&0u16.to_be_bytes()) {
                            match e.kind() {
                                io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset => return,
                                _ => {
                                    eprintln!("TCP write empty failed to {}: {}", peer, e);
                                    return;
                                }
                            }
                        }
                    }
                }
            } else {
                eprintln!("No engine configured; closing TCP connection from {}", peer);
                return;
            }
        }
    }

    pub fn run(&self) -> io::Result<()> {
        let bind_addr = self.config.bind_addr.clone();
        let port = self.config.port;

        let udp_bind = format!("{}:{}", bind_addr, port);
        let udp_socket = UdpSocket::bind(&udp_bind)?;
        udp_socket.set_nonblocking(true)?;

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
                            let send_sock = sock.clone();
                            let engine = udp_engine.clone();
                            // increment UDP metric asynchronously
                            if let Ok(db_url) = std::env::var("DATABASE_URL") {
                                let _ = thread::spawn(move || {
                                    if let Ok(mut client) = Client::connect(&db_url, NoTls) {
                                        let _ = client.execute("INSERT INTO dns_query_metrics (id, udp_count, tcp_count, inserted_at, updated_at) VALUES (1, 1, 0, now(), now()) ON CONFLICT (id) DO UPDATE SET udp_count = dns_query_metrics.udp_count + 1, updated_at = now()", &[]);
                                    }
                                });
                            }

                            thread::spawn(move || {
                                if let Some(engine) = &engine {
                                    if let Some(resp) = engine.handle_query(&req) {
                                        let _ = send_sock.send_to(&resp, src);
                                    }
                                }
                            });
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(1));
                        }
                        Err(e) => {
                            eprintln!("UDP recv error: {}", e);
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
                        let peer = s
                            .peer_addr()
                            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                        let eng = tcp_engine.clone();
                        thread::spawn(move || {
                            Nameserver::handle_tcp_connection(s, eng, peer);
                        });
                    }
                    Err(e) => {
                        eprintln!("TCP accept error: {}", e);
                    }
                }
            }
            Ok(())
        });

        let _ = udp_handle.join();
        let _ = tcp_handle.join();
        Ok(())
    }
}
