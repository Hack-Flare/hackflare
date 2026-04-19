use crate::ns::NsConfig;
use std::io;
use std::net::UdpSocket;

pub struct Nameserver {
    pub config: NsConfig,
}

impl Nameserver {
    pub fn new(config: NsConfig) -> Self {
        Self { config }
    }

    pub fn run(&self) -> io::Result<()> {
        let bind = format!("{}:{}", self.config.bind_addr, self.config.port);
        let socket = UdpSocket::bind(&bind)?;
        socket.set_nonblocking(false)?;
        let mut buf = [0u8; 512];
        loop {
            let (amt, src) = socket.recv_from(&mut buf)?;
            eprintln!("Received {} bytes from {} (stub)", amt, src);

            let _ = socket.send_to(&buf[..amt.min(buf.len())], src);
        }
    }
}
