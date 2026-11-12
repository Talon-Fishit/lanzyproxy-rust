//! The ENet relay.
//!
//! The client connects here; the proxy opens a second connection to the real
//! server and shuttles packets between the two, watching the traffic in both
//! directions so commands and automation can act on it.

use std::net::SocketAddr;
use std::time::Duration;

use enet::{Address, BandwidthLimit, ChannelLimit, Enet, Event, Host, Packet, PacketMode, Peer};

use lzconfig::Config;
use lzfeatures::Context;
use lzprotocol::{GameUpdatePacket, NetMessage, Tank, VariantList};

/// Peer tag: the game client.
pub const TAG_CLIENT: u32 = 1;
/// Peer tag: the upstream server.
pub const TAG_UPSTREAM: u32 = 2;

/// Which side of the relay a packet came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// From the game client.
    Client,
    /// From the upstream server.
    Server,
}

/// What to do with a packet the proxy intercepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Let it through unchanged.
    Pass,
    /// Drop it.
    Drop,
}

/// Inspect one relayed packet. Returns what to do with it.
pub fn inspect(ctx: &mut Context, side: Side, data: &[u8]) -> Decision {
    let (kind, payload) = lzprotocol::unframe(data);
    match kind {
        NetMessage::GenericText => {
            let text = lzprotocol::parse_text_payload(payload);
            // A chat line that starts with "/" is a command for us.
            if side == Side::Client && lzfeatures::is_command(&text) {
                lzfeatures::dispatch(ctx, &text);
                return Decision::Drop;
            }
            // A button press on one of our own pages.
            if side == Side::Client {
                let (action, fields) = lzprotocol::parse_action(&text);
                if action == "dialog_return" {
                    for (key, value) in &fields {
                        if key == "button" && lzfeatures::options_button(ctx, value) {
                            return Decision::Drop;
                        }
                    }
                }
            }
            // Roulette announcements are decorated on their way through.
            if side == Side::Server && lzfeatures::casino::on_talk(ctx, &text, "server").is_some() {
                return Decision::Drop;
            }
            Decision::Pass
        }
        NetMessage::GamePacket => {
            if payload.len() < lzprotocol::GUP_SIZE {
                return Decision::Pass;
            }
            let Ok(pkt) = GameUpdatePacket::parse(payload) else {
                return Decision::Pass;
            };
            let tank = pkt.packet_type as i32;
            if tank == Tank::SendMapData as i32 || tank == Tank::SendMapDataV2 as i32 {
                lzfeatures::worldtools::decode_world_data(ctx, &payload[lzprotocol::GUP_SIZE..]);
            } else if tank == Tank::CallFunction as i32 {
                // Keep the function list; the game expects it untouched.
                let _ = VariantList::parse(&payload[lzprotocol::GUP_SIZE..]);
            }
            Decision::Pass
        }
        _ => Decision::Pass,
    }
}

/// The address the proxy listens on.
pub fn listen_address(config: &Config) -> SocketAddr {
    let host = config.get_str("proxy_host");
    let port = config.get_int("proxy_port") as u16;
    format!("{host}:{port}")
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], port)))
}

/// Resolve a host name to an ENet address.
pub fn resolve(host: &str, port: u16) -> Option<Address> {
    use std::net::ToSocketAddrs;
    let addrs = (host, port).to_socket_addrs().ok()?;
    let first = addrs.into_iter().next()?;
    match first {
        std::net::SocketAddr::V4(v4) => Some(Address::new(*v4.ip(), port)),
        std::net::SocketAddr::V6(_) => None,
    }
}

/// How long to wait for ENet events.
pub const SERVICE_TIMEOUT: Duration = Duration::from_millis(50);

/// What the relay should do after one service call.
enum Action {
    /// Nothing happened.
    Idle,
    /// The game client connected.
    ClientConnected,
    /// A peer went away.
    Disconnected(u32),
    /// A packet arrived.
    Receive {
        from: u32,
        channel_id: u8,
        data: Vec<u8>,
    },
}

/// Find the peer carrying a tag.
fn peer_with_tag<'h>(
    host: &'h mut Host<u32>,
    tag: u32,
) -> Option<Peer<'h, u32>> {
    host.peers().find(|p| p.data().copied() == Some(tag))
}

/// Run the relay until the client disconnects.
///
/// The ENet host owns its peers and its events borrow it, so the whole loop
/// lives in one function: an event is reduced to a plain `Action` and dropped
/// before the host is touched again.
pub fn run(config: &Config, mut ctx: Context, log: &dyn Fn(&str)) -> Result<(), String> {
    let enet = Enet::new().map_err(|e| format!("enet init failed: {e}"))?;
    let listen = listen_address(config);
    // ENet addresses are IPv4.
    let ipv4 = match listen.ip() {
        std::net::IpAddr::V4(v4) => v4,
        std::net::IpAddr::V6(v6) => v6.to_ipv4_mapped().unwrap_or([127, 0, 0, 1].into()),
    };
    let address = Address::new(ipv4, listen.port());

    let mut host = enet
        .create_host::<u32>(
            Some(&address),
            32,
            ChannelLimit::Limited(2),
            BandwidthLimit::Unlimited,
            BandwidthLimit::Unlimited,
        )
        .map_err(|e| format!("cannot bind the proxy port: {e}"))?;

    log(&format!("[enet] listening on {listen}"));

    let upstream_host = config.get_str("upstream_host");
    let upstream_port = config.get_int("upstream_port") as u16;
    let mut need_upstream = false;

    loop {
        let action = {
            let maybe_event = match host.service(SERVICE_TIMEOUT.as_millis() as u32) {
                Ok(e) => e,
                Err(e) => return Err(format!("enet service failed: {e}")),
            };
            let Some(mut event) = maybe_event else {
                // No event: the automation tickers run on the idle path.
                lzfeatures::tick(&mut ctx);
                continue;
            };
            match &mut event {
                Event::Connect(peer) => {
                    // A peer that arrives on its own is the game client.
                    peer.set_data(Some(TAG_CLIENT));
                    Action::ClientConnected
                }
                Event::Disconnect(peer, _) => {
                    Action::Disconnected(peer.data().copied().unwrap_or(0))
                }
                Event::Receive {
                    sender,
                    channel_id,
                    packet,
                } => Action::Receive {
                    from: sender.data().copied().unwrap_or(0),
                    channel_id: *channel_id,
                    data: packet.data().to_vec(),
                },
            }
        };

        match action {
            Action::Idle => {
                lzfeatures::tick(&mut ctx);
            }
            Action::ClientConnected => {
                ctx.state.connected = true;
                log("[enet] client connected");
                need_upstream = true;
            }
            Action::Disconnected(tag) => {
                log(&format!("[enet] peer {tag} disconnected"));
                if tag == TAG_CLIENT {
                    ctx.state.connected = false;
                    ctx.state.reset_session();
                    need_upstream = false;
                }
            }
            Action::Receive {
                from,
                channel_id,
                data,
            } => {
                let side = if from == TAG_CLIENT {
                    Side::Client
                } else {
                    Side::Server
                };
                let decision = inspect(&mut ctx, side, &data);
                if decision == Decision::Drop {
                    continue;
                }
                let target = if from == TAG_CLIENT {
                    TAG_UPSTREAM
                } else {
                    TAG_CLIENT
                };
                if let Some(mut peer) = peer_with_tag(&mut host, target) {
                    if let Ok(p) = Packet::new(&data, PacketMode::ReliableSequenced) {
                        let _ = peer.send_packet(p, channel_id);
                    }
                }
            }
        }

        if need_upstream {
            need_upstream = false;
            let Some(addr) = resolve(&upstream_host, upstream_port) else {
                log(&format!("[enet] cannot resolve {upstream_host}"));
                return Err(format!("cannot resolve {upstream_host}"));
            };
            match host.connect(&addr, 2, 0) {
                Ok(mut peer) => {
                    peer.set_data(Some(TAG_UPSTREAM));
                    log(&format!("[enet] upstream {upstream_host}:{upstream_port}"));
                }
                Err(e) => {
                    log(&format!("[enet] cannot reach the upstream: {e}"));
                    return Err(format!("cannot reach the upstream: {e}"));
                }
            }
        }
    }
}
