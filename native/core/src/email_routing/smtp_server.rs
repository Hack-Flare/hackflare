use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

use crate::email_routing::sending::send_email_smtp;

fn mx_hosts_for(domain: &str) -> Result<Vec<String>, String> {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())
        .map_err(|e| format!("resolver init error: {}", e))?;
    let mx = resolver
        .mx_lookup(domain)
        .map_err(|e| format!("mx lookup failed: {}", e))?;
    let mut hosts = vec![];
    for r in mx.iter() {
        hosts.push(r.exchange().to_utf8());
    }
    Ok(hosts)
}

fn handle_client(mut stream: TcpStream) -> Result<(), String> {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_default();
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut line = String::new();
    let mut mail_from = String::new();
    let mut rcpt_to: Vec<String> = vec![];
    let mut data_mode = false;
    let mut data_buf = String::new();

    let mut write_ok = |s: &str| -> Result<(), String> {
        stream.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;
        Ok(())
    };

    write_ok("220 hackflare SMTP ready\r\n")?;

    loop {
        line.clear();
        let n = reader.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let cmd = line.trim_end_matches(&['\r', '\n'][..]).to_string();
        if data_mode {
            if cmd == "." {
                data_mode = false;

                let raw = data_buf.clone();
                for rcpt in &rcpt_to {
                    if let Some(pos) = rcpt.rfind('@') {
                        let domain = &rcpt[pos + 1..];
                        match mx_hosts_for(domain) {
                            Ok(hosts) if !hosts.is_empty() => {
                                let host = &hosts[0];

                                let from = if mail_from.starts_with('<') && mail_from.ends_with('>')
                                {
                                    mail_from
                                        .trim_start_matches('<')
                                        .trim_end_matches('>')
                                        .to_string()
                                } else {
                                    mail_from.clone()
                                };
                                let _ = send_email_smtp(
                                    rcpt,
                                    &from,
                                    "",
                                    &raw,
                                    Some(host.as_str()),
                                    None,
                                    None,
                                );
                            }
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("MX lookup failed for {}: {}", rcpt, e);
                            }
                        }
                    }
                }
                data_buf.clear();
                write_ok("250 Ok\r\n")?;
            } else {
                data_buf.push_str(&line);
            }
            continue;
        }

        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let verb = parts[0].to_uppercase();
        match verb.as_str() {
            "HELO" | "EHLO" => {
                write_ok("250 Hello\r\n")?;
            }
            "MAIL" => {
                if let Some(p) = parts.get(1) {
                    mail_from = p.trim().to_string();
                }
                write_ok("250 OK\r\n")?;
            }
            "RCPT" => {
                if let Some(p) = parts.get(1) {
                    rcpt_to.push(p.trim().to_string());
                }
                write_ok("250 OK\r\n")?;
            }
            "DATA" => {
                data_mode = true;
                write_ok("354 End data with <CR><LF>.<CR><LF>\r\n")?;
            }
            "RSET" => {
                mail_from.clear();
                rcpt_to.clear();
                data_buf.clear();
                write_ok("250 OK\r\n")?;
            }
            "QUIT" => {
                write_ok("221 Bye\r\n")?;
                break;
            }
            _ => {
                write_ok("502 Command not implemented\r\n")?;
            }
        }
    }

    eprintln!("Connection {} closed", peer);
    Ok(())
}

pub fn start_smtp_server(bind_addr: &str) -> Result<(), String> {
    let listener = TcpListener::bind(bind_addr).map_err(|e| format!("bind error: {}", e))?;
    eprintln!("SMTP server listening on {}", bind_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(|| {
                    if let Err(e) = handle_client(s) {
                        eprintln!("client handler error: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("incoming connection failed: {}", e),
        }
    }
    Ok(())
}

#[rustler::nif]
pub fn start_smtp_server_nif(bind: String) -> Result<(), String> {
    let bind2 = bind.clone();
    thread::spawn(move || {
        if let Err(e) = start_smtp_server(&bind2) {
            eprintln!("smtp server error: {}", e);
        }
    });
    Ok(())
}
