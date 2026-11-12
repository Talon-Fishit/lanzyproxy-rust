//! A UDP relay that carries the game's traffic through a SOCKS5 proxy.
//!
//! The reference build lets the player point the proxy at a SOCKS5 server and
//! sends the game's UDP traffic through it, because the game blocks an IP that
//! plays too much. It reports a failure when "Your Socks5 Doesn't Support UDP".
//!
//! ENet cannot speak SOCKS5 itself, so this runs a small UDP forwarder in
//! front of it: ENet connects to `127.0.0.1:<local port>`, and this relays
//! every datagram to the real game server through the SOCKS5 proxy's UDP
//! ASSOCIATE tunnel. ENet is unaware of the difference.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

/// SOCKS5 request codes.
const CMD_UDP_ASSOCIATE: u8 = 0x03;
const ATYP_IPV4: u8 = 0x01;
const ATYP_DOMAIN: u8 = 0x03;

/// A configured SOCKS5 proxy.
#[derive(Debug, Clone, Default)]
pub struct Socks5 {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
}

impl Socks5 {
    /// Whether a proxy is configured.
    pub fn is_set(&self) -> bool {
        !self.host.is_empty() && self.port != 0
    }
}

/// Open a SOCKS5 UDP ASSOCIATE tunnel and return the relay endpoint.
fn udp_associate(proxy: &Socks5) -> std::io::Result<SocketAddr> {
    let mut stream = std::net::TcpStream::connect((proxy.host.as_str(), proxy.port))?;
    stream.set_read_timeout(Some(Duration::from_secs(15)))?;
    stream.set_write_timeout(Some(Duration::from_secs(15)))?;

    // Greeting: offer no-auth and username/password.
    let mut greeting = vec![0x05, 0x02, 0x00, 0x02];
    stream.write_all(&greeting)?;

    let mut reply = [0u8; 2];
    stream.read_exact(&mut reply)?;
    if reply[0] != 0x05 {
        return Err(std::io::Error::other("not a SOCKS5 server"));
    }

    match reply[1] {
        0x00 => {}
        0x02 => {
            // Username/password authentication.
            let mut auth = vec![0x01, proxy.user.len() as u8];
            auth.extend_from_slice(proxy.user.as_bytes());
            auth.push(proxy.pass.len() as u8);
            auth.extend_from_slice(proxy.pass.as_bytes());
            stream.write_all(&auth)?;
            let mut ok = [0u8; 2];
            stream.read_exact(&mut ok)?;
            if ok[1] != 0x00 {
                return Err(std::io::Error::other("SOCKS5 authentication failed"));
            }
        }
        _ => return Err(std::io::Error::other("SOCKS5 wants an unsupported auth method")),
    }

    // The UDP request carries the address the datagrams will be sent from. A
    // zero address lets the proxy pick.
    let mut request = vec![0x05, CMD_UDP_ASSOCIATE, 0x00, ATYP_IPV4];
    request.extend_from_slice(&[0, 0, 0, 0]);
    request.extend_from_slice(&0u16.to_be_bytes());
    stream.write_all(&request)?;

    let mut head = [0u8; 4];
    stream.read_exact(&mut head)?;
    if head[1] != 0x00 {
        return Err(std::io::Error::other(format!(
            "SOCKS5 UDP associate refused: {}",
            head[1]
        )));
    }
    let mut relay = Vec::new();
    match head[3] {
        ATYP_IPV4 => {
            let mut buf = [0u8; 6];
            stream.read_exact(&mut buf)?;
            relay.extend_from_slice(&buf);
        }
        ATYP_DOMAIN => {
            let mut len = [0u8; 1];
            stream.read_exact(&mut len)?;
            let mut name = vec![0u8; len[0] as usize];
            stream.read_exact(&mut name)?;
            let mut port = [0u8; 2];
            stream.read_exact(&mut port)?;
            let host = String::from_utf8_lossy(&name).into_owned();
            let addrs = (host.as_str(), u16::from_be_bytes(port)).to_socket_addrs()?;
            let first = addrs
                .into_iter()
                .next()
                .ok_or_else(|| std::io::Error::other("SOCKS5 gave no relay address"))?;
            return Ok(first);
        }
        other => {
            return Err(std::io::Error::other(format!(
                "SOCKS5 relay address type {other} is not supported"
            )))
        }
    }
    let ip = Ipv4Addr::new(relay[0], relay[1], relay[2], relay[3]);
    let port = u16::from_be_bytes([relay[4], relay[5]]);
    Ok(SocketAddr::from((ip, port)))
}

/// A running UDP relay.
pub struct Relay {
    /// The local address ENet should connect to.
    pub local: SocketAddr,
    /// The address the game server is really at.
    pub target: SocketAddr,
}

/// Start the relay: bind a local UDP socket, open the SOCKS5 tunnel, and pump
/// datagrams between the two.
pub fn start(
    proxy: &Socks5,
    target_host: &str,
    target_port: u16,
) -> std::io::Result<Relay> {
    let target: SocketAddr = (target_host, target_port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| std::io::Error::other("cannot resolve the game server"))?;

    let assoc = udp_associate(proxy)?;
    // The SOCKS5 server wants the source address we will send from, so the
    // control socket has to stay open for the life of the tunnel.
    let _control = std::net::TcpStream::connect((proxy.host.as_str(), proxy.port))?;

    let local = UdpSocket::bind("127.0.0.1:0")?;
    let local_addr = local.local_addr()?;
    let to_proxy = UdpSocket::bind("0.0.0.0:0")?;

    let up = local.try_clone()?;
    let down = to_proxy.try_clone()?;
    let target_addr = target;
    let assoc_addr = assoc;

    // Local -> SOCKS5 -> game server.
    std::thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            let Ok((n, _)) = up.recv_from(&mut buf) else { continue };
            // SOCKS5 UDP datagram: RSV(2) FRAG(1) ATYP(1) ADDR PORT DATA
            let mut framed = Vec::with_capacity(n + 22);
            framed.extend_from_slice(&[0x00, 0x00, 0x00]);
            match target_addr {
                SocketAddr::V4(v4) => {
                    framed.push(ATYP_IPV4);
                    framed.extend_from_slice(&v4.ip().octets());
                    framed.extend_from_slice(&v4.port().to_be_bytes());
                }
                SocketAddr::V6(v6) => {
                    framed.push(0x04);
                    framed.extend_from_slice(&v6.ip().octets());
                    framed.extend_from_slice(&v6.port().to_be_bytes());
                }
            }
            framed.extend_from_slice(&buf[..n]);
            let _ = to_proxy.send_to(&framed, assoc_addr);
        }
    });

    // SOCKS5 -> local.
    std::thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            let Ok((n, _)) = down.recv_from(&mut buf) else { continue };
            if n < 10 {
                continue;
            }
            // Skip RSV(2) FRAG(1) ATYP(1), then the address.
            let atyp = buf[3];
            let payload_at = match atyp {
                ATYP_IPV4 => 4 + 4 + 2,
                ATYP_DOMAIN => 4 + 1 + buf[4] as usize + 2,
                0x04 => 4 + 16 + 2,
                _ => continue,
            };
            if n <= payload_at {
                continue;
            }
            let _ = local.send_to(&buf[payload_at..], local_addr);
        }
    });

    Ok(Relay { local: local_addr, target })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unset_proxy_is_not_used() {
        let proxy = Socks5::default();
        assert!(!proxy.is_set());
        let proxy = Socks5 {
            host: "1.2.3.4".into(),
            port: 1080,
            ..Default::default()
        };
        assert!(proxy.is_set());
    }
}
