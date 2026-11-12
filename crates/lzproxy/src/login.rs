//! Logging in, and pointing the game client at this proxy.
//!
//! The reference build does two things the port had to learn about:
//!
//! 1. After the upstream connection is up it sends its own login request:
//!    `action|logon` with the `logon|`, `password|` and `password_verify|`
//!    fields.
//! 2. It edits the Windows hosts file so `www.growtopia1.com` and
//!    `www.growtopia2.com` resolve to the proxy, which is how the game client
//!    ends up talking here without any change to the client itself.

use std::fs;
use std::path::{Path, PathBuf};

/// The marker the reference leaves next to its hosts entries.
const HOSTS_MARKER: &str = "# added by LanzyProxy, Do not edit.";

/// Build the login request the proxy sends upstream.
pub fn login_request(username: &str, password: &str) -> String {
    format!("action|logon\n|logon|{username}\n|password|{password}\n|password_verify|{password}")
}

/// The hosts file the reference writes to.
pub fn hosts_path() -> std::path::PathBuf {
    if cfg!(target_os = "windows") {
        let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        return Path::new(&root).join("System32/drivers/etc/hosts");
    }
    PathBuf::from("/etc/hosts")
}

/// Point the game server names at the local proxy.
///
/// The entries are marked so they can be removed again later.
pub fn write_hosts(proxy_ip: &str, servers: &[&str]) -> std::io::Result<()> {
    let path = hosts_path();
    let existing = fs::read_to_string(&path).unwrap_or_default();

    // Drop any entry this tool wrote before.
    let kept: Vec<&str> = existing
        .lines()
        .filter(|line| !line.contains(HOSTS_MARKER))
        .collect();

    let mut out = String::new();
    for line in kept {
        out.push_str(line);
        out.push('\n');
    }
    for server in servers {
        out.push_str(&format!(" {proxy_ip} {server}   {HOSTS_MARKER}\n"));
    }
    fs::write(&path, out)
}

/// Remove the entries this tool wrote.
pub fn clear_hosts() -> std::io::Result<()> {
    let path = hosts_path();
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let kept: Vec<&str> = existing
        .lines()
        .filter(|line| !line.contains(HOSTS_MARKER))
        .collect();
    let mut out = String::new();
    for line in kept {
        out.push_str(line);
        out.push('\n');
    }
    fs::write(&path, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_login_request_carries_every_field() {
        let request = login_request("player", "secret");
        assert!(request.starts_with("action|logon"));
        assert!(request.contains("|logon|player"));
        assert!(request.contains("|password|secret"));
        assert!(request.contains("|password_verify|secret"));
    }
}
