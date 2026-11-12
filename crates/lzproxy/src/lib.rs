//! The proxy: an HTTP spoofer for `server_data.php` and the ENet relay.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use lzconfig::Config;
use lzfeatures::Context;
use lzstate::State;

pub mod enet;
pub mod spoofer;

/// Where a proxied session's state and packets are shared.
pub struct Session {
    pub context: Mutex<Context>,
    /// Packets queued for the client.
    pub to_client: Mutex<Vec<Vec<u8>>>,
}

/// The body the spoofer answers `server_data.php` with.
pub fn server_data_body(config: &Config) -> String {
    let host = config.get_str("proxy_host");
    let port = config.get_int("proxy_port");
    format!(
        "server|{host}\n\
         port|{port}\n\
         type2|1\n\
         meta|lanzyproxy\n\
         beta_server|{host}\n\
         beta_port|{port}\n\
         beta_type2|1\n\
         beta_meta|lanzyproxy-beta\n\
         RTENDMARKERBS1001"
    )
}

/// Read one HTTP request from a socket, returning the path.
pub fn read_request_path(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    let text = String::from_utf8_lossy(&buf[..n]);
    Ok(text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
        .to_string())
}

/// Answer one HTTP request on the spoofer socket.
pub fn serve_spoof_request(stream: &mut TcpStream, config: &Config) -> std::io::Result<()> {
    let path = read_request_path(stream)?;
    let (status, body, ctype) = if path.contains("/growtopia/server_data.php") {
        ("200 OK", server_data_body(config), "text/html")
    } else if path.contains("/growtopia/server_data.php") {
        ("200 OK", "lanzyproxy".to_string(), "text/plain")
    } else {
        ("404 Not Found", String::new(), "text/plain")
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())
}

/// Start the `server_data.php` spoofer. Returns the bound port.
pub fn start_spoofer(config: Arc<Config>, log: Arc<dyn Fn(&str) + Send + Sync>) -> std::io::Result<u16> {
    let port = config.get_int("spoof_http_port") as u16;
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    let bound = listener.local_addr()?.port();
    log(&format!(
        "[http] server_data.php spoofer on port {bound} -> {}:{}",
        config.get_str("proxy_host"),
        config.get_int("proxy_port")
    ));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let config = config.clone();
            let log = log.clone();
            std::thread::spawn(move || {
                let _ = serve_spoof_request(&mut stream, &config);
                let _ = log;
            });
        }
    });
    Ok(bound)
}

/// A shared handle to the running proxy.
pub type Shared = Arc<Session>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_data_points_at_the_proxy() {
        let mut config = Config::ephemeral();
        config.set_str("proxy_host", "10.0.0.5");
        config.set_int("proxy_port", 17198);
        let body = server_data_body(&config);
        assert!(body.contains("server|10.0.0.5"));
        assert!(body.contains("port|17198"));
        assert!(body.trim_end().ends_with("RTENDMARKERBS1001"));
    }
}
