//! The HTTP spoofer module.
//!
//! Growtopia's client asks `server_data.php` where to connect. Answering it
//! with the local proxy's address is what puts the client's traffic through
//! this process.

use std::io::Write;
use std::net::TcpStream;

/// Write a raw HTTP response to a socket.
pub fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())
}

/// A 404 for any path the spoofer does not own.
pub fn write_not_found(stream: &mut TcpStream) -> std::io::Result<()> {
    write_response(stream, "404 Not Found", "text/plain", "")
}
