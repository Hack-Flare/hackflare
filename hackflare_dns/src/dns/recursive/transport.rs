use crate::dns::DnsConfig;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream, UdpSocket};
use std::time::{Duration, Instant};

use super::message::response_matches_expected;

fn socket_target(addr: &str) -> String {
    if addr.contains(':') && !addr.starts_with('[') {
        format!("[{addr}]:53")
    } else {
        format!("{addr}:53")
    }
}

pub(crate) struct UdpTransport {
    sock: UdpSocket,
}

impl UdpTransport {
    pub(crate) fn bind(timeout: Duration) -> Option<Self> {
        let sock = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
        let _ = sock.set_read_timeout(Some(timeout));
        Some(Self { sock })
    }

    pub(crate) fn send_recv(
        &self,
        addr: &str,
        msg: &[u8],
        qid: u16,
        qname: &str,
        qtype: u16,
        config: &DnsConfig,
    ) -> Option<Vec<u8>> {
        let target = socket_target(addr);
        let expected_ip: IpAddr = addr.parse().ok()?;
        let mut buf = [0u8; 4096];
        let attempts = config.udp_attempts.max(1);
        let timeout = config.udp_timeout;

        for _ in 0..attempts {
            let _ = self.sock.send_to(msg, &target).ok()?;
            let deadline = Instant::now() + timeout;

            loop {
                if Instant::now() >= deadline {
                    break;
                }
                match self.sock.recv_from(&mut buf) {
                    Ok((amt, src)) => {
                        if src.port() != 53 || src.ip() != expected_ip {
                            continue;
                        }
                        let candidate = &buf[..amt];
                        if response_matches_expected(candidate, qid, qname, qtype) {
                            return Some(candidate.to_vec());
                        }
                    }
                    Err(ref e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        break;
                    }
                    Err(_) => break,
                }
            }
        }

        None
    }
}

pub(super) fn tcp_send_recv(addr: &str, msg: &[u8]) -> Option<Vec<u8>> {
    let target = socket_target(addr);
    let sockaddr = target.parse().ok()?;
    let mut stream = TcpStream::connect_timeout(&sockaddr, Duration::from_secs(3)).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(4))).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(4)))
        .ok()?;
    let len = u16::try_from(msg.len()).unwrap_or(0).to_be_bytes();
    if stream.write_all(&len).is_err() {
        return None;
    }
    if stream.write_all(msg).is_err() {
        return None;
    }
    let mut lenbuf = [0u8; 2];
    if stream.read_exact(&mut lenbuf).is_err() {
        return None;
    }
    let rlen = u16::from_be_bytes(lenbuf) as usize;
    let mut buf = vec![0u8; rlen];
    if stream.read_exact(&mut buf).is_err() {
        return None;
    }
    Some(buf)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn socket_target_formats_ipv4_and_ipv6() {
        assert_eq!(socket_target("192.0.2.1"), "192.0.2.1:53");
        assert_eq!(socket_target("2001:db8::1"), "[2001:db8::1]:53");
    }
}
