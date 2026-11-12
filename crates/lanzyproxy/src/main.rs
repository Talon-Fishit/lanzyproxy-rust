//! The `lanzyproxy` binary.
//!
//! Three modes:
//!   lanzyproxy              run the proxy (spoofer + ENet relay)
//!   lanzyproxy --offline    the offline shell: run commands without a game
//!   lanzyproxy --list       print the command list
//!   lanzyproxy --version    print the version

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use lzconfig::Config;
use lzfeatures::Context;
use lzproxy::spoofer;
use lzstate::State;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the config file lives.
fn config_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("LANZYPROXY_CONFIG") {
        return PathBuf::from(explicit);
    }
    let exe = std::env::current_exe().ok();
    let base = exe
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("config.json")
}

fn banner() {
    println!("LanzyProxy {VERSION} - Growtopia proxy");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("lanzyproxy {VERSION}");
        return;
    }

    if args.iter().any(|a| a == "--list" || a == "--commands") {
        let mut names: Vec<&str> = lzfeatures::REGISTRY.keys().copied().collect();
        names.sort_unstable();
        for name in names.iter() {
            println!("  /{name:<14} {}", lzfeatures::REGISTRY[*name].summary);
        }
        println!("\n{} commands", names.len());
        return;
    }

    if args.iter().any(|a| a == "--offline") {
        offline();
        return;
    }

    banner();
    let path = config_path();
    let config = Config::load(&path);
    println!("[config] {}", path.display());

    let log = Arc::new(|line: &str| println!("{line}"));

    let config = Arc::new(config);
    let spoofer_port = match lzproxy::start_spoofer(config.clone(), log.clone()) {
        Ok(port) => Some(port),
        Err(e) => {
            log(&format!("[http] spoofer failed: {e}"));
            None
        }
    };
    let _ = spoofer_port;

    let ctx = Context::new(State::new(), (*config).clone());
    let listen = lzproxy::enet::listen_address(&config);
    println!("[proxy] relaying {listen} -> {}:{}", config.get_str("upstream_host"), config.get_int("upstream_port"));

    if let Err(e) = lzproxy::enet::run(&config, ctx, &|line| println!("{line}")) {
        eprintln!("[proxy] {e}");
        std::process::exit(1);
    }
}

/// The offline shell: type commands and see what would be sent.
fn offline() {
    banner();
    println!("offline shell - type a command, or /quit to leave");
    let config = Config::ephemeral();
    let mut ctx = Context::new(State::new(), config);
    let queued = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));

    {
        let sink = queued.clone();
        ctx.attach_sink(Box::new(move |data: &[u8]| {
            sink.lock().unwrap().push(data.to_vec());
        }));
    }

    let mut line = String::new();
    loop {
        print!("> ");
        use std::io::Write;
        let _ = std::io::stdout().flush();
        line.clear();
        if std::io::stdin().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let text = line.trim();
        if text.is_empty() {
            continue;
        }
        if text == "/quit" || text == "/exit" {
            break;
        }
        if lzfeatures::dispatch(&mut ctx, text) {
            let packets = {
                let mut guard = queued.lock().unwrap();
                std::mem::take(&mut *guard)
            };
            for packet in packets {
                println!("  -> {} bytes: {}", packet.len(), preview(&packet));
            }
        }
    }
}

/// A short, readable view of a packet.
fn preview(packet: &[u8]) -> String {
    let (kind, payload) = lzprotocol::unframe(packet);
    match kind {
        lzprotocol::NetMessage::GenericText => {
            format!("text {:?}", lzprotocol::parse_text_payload(payload))
        }
        lzprotocol::NetMessage::GamePacket => {
            if payload.len() >= lzprotocol::GUP_SIZE {
                if let Ok(pkt) = lzprotocol::GameUpdatePacket::parse(payload) {
                    return format!("game packet type={} netid={}", pkt.packet_type, pkt.netid);
                }
            }
            format!("game packet ({} bytes)", payload.len())
        }
        other => format!("message {other:?}"),
    }
}
