//! Command registry and the glue between features and the proxy.
//!
//! Every command lives in one of the feature modules; importing them here
//! registers them exactly once. The registry maps a name and its aliases to a
//! handler, and `dispatch` runs a chat line against it.

use std::collections::HashMap;
use std::sync::LazyLock;

use lzconfig::Config;
use lzdialog::Dialog;
use lzprotocol::{
    action_packet, console_message, dialog_request, game_packet, variant_packet, GameUpdatePacket,
    NetMessage, Tank, VariantList,
};
use lzstate::State;

pub mod automation;
pub mod casino;
pub mod commands;
pub mod common;
pub mod economy;
pub mod moderation;
pub mod navigation;
pub mod options;
pub mod utility;
pub mod visuals;
pub mod worldtools;

/// Everything a feature is allowed to touch.
pub struct Context {
    pub state: State,
    pub config: Config,
    sink: Option<Box<dyn FnMut(&[u8]) + Send>>,
    logger: Option<Box<dyn Fn(&str) + Send + Sync>>,
    send_stack: Vec<Vec<u8>>,
}

impl Context {
    /// A context with no sink: outgoing packets queue in `send_stack`.
    pub fn new(state: State, config: Config) -> Context {
        Context {
            state,
            config,
            sink: None,
            logger: None,
            send_stack: Vec::new(),
        }
    }

    /// Attach the function that writes packets to the client.
    pub fn attach_sink(&mut self, sink: Box<dyn FnMut(&[u8]) + Send>) {
        self.sink = Some(sink);
    }

    /// Attach the function that reports lines to the operator.
    pub fn attach_logger(&mut self, logger: Box<dyn Fn(&str) + Send + Sync>) {
        self.logger = Some(logger);
    }

    /// Queue or send one packet.
    pub fn send_raw(&mut self, payload: &[u8]) {
        match self.sink.as_mut() {
            Some(sink) => sink(payload),
            None => self.send_stack.push(payload.to_vec()),
        }
    }

    /// Take the packets queued while there was no sink.
    pub fn take_queued(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.send_stack)
    }

    /// Record a line in the session log.
    pub fn log(&mut self, message: &str) {
        self.state.log(message);
        if let Some(logger) = self.logger.as_ref() {
            logger(message);
        }
    }

    // -- primitives --------------------------------------------------------

    /// Print a line in the game chat.
    pub fn console(&mut self, text: &str) {
        self.send_raw(&console_message(text));
    }

    /// Open a dialog.
    pub fn dialog(&mut self, name: &str, dialog: &Dialog) {
        let markup = dialog.build("OK", "Cancel");
        self.send_raw(&dialog_request(name, &markup));
    }

    /// Start a dialog page.
    pub fn page(&self, name: &str, title: &str) -> Dialog {
        Dialog::titled(name, title)
    }

    /// Send a client-side `action|...` message.
    pub fn input(&mut self, text: &str) {
        self.send_raw(&action_packet(text));
    }

    /// Send a `CALL_FUNCTION` packet.
    pub fn variant(&mut self, func: &str, params: &[lzprotocol::Variant]) {
        let mut list = VariantList::with_func(func);
        for p in params {
            list.add(p.clone());
        }
        self.send_raw(&variant_packet(&list));
    }

    /// Answer a dialog the client closed.
    pub fn dialog_return(&mut self, fields: &[(&str, String)]) {
        let mut parts: Vec<String> = vec!["action|dialog_return".into()];
        for (k, v) in fields {
            parts.push(format!("|{k}|{v}"));
        }
        self.input(&parts.join("\n"));
    }

    /// Send a `GAME_PACKET` with the given header.
    pub fn state_packet(&mut self, pkt: &GameUpdatePacket) {
        self.send_raw(&game_packet(pkt, &[]));
    }

    /// Move the player.
    pub fn move_to(&mut self, x: f32, y: f32) {
        let pkt = GameUpdatePacket {
            packet_type: Tank::State as u8,
            netid: self.state.netid,
            x,
            y,
            xpos: (x as i64 / 32) as f32,
            ypos: (y as i64 / 32) as f32,
            ..Default::default()
        };
        self.state_packet(&pkt);
    }

    /// Punch a tile.
    pub fn punch(&mut self, tile_x: i32, tile_y: i32) {
        let pkt = GameUpdatePacket {
            packet_type: Tank::TileChangeRequest as u8,
            netid: self.state.netid,
            x: self.state.pos_x,
            y: self.state.pos_y,
            tile_x,
            tile_y,
            ..Default::default()
        };
        self.state_packet(&pkt);
    }

    /// Place an item on a tile.
    pub fn place(&mut self, tile_x: i32, tile_y: i32, item_id: i32, layer: u8) {
        let mut pkt = GameUpdatePacket {
            packet_type: Tank::TileChangeRequest as u8,
            netid: self.state.netid,
            tile_x,
            tile_y,
            ..Default::default()
        };
        pkt.extra[0] = layer;
        self.state_packet(&pkt);
        self.input(&format!(
            "action|place\n|itemID|{item_id}\n|tileX|{tile_x}\n|tileY|{tile_y}"
        ));
    }

    /// Say something in chat.
    pub fn say(&mut self, text: &str) {
        self.input(&format!("action|input\n|text|{text}"));
    }

    /// Join a world.
    pub fn warp(&mut self, world: &str) {
        self.input(&format!("action|join_request\n|name|{world}\n|invis|0"));
    }

    /// Use the wrench on a tile.
    pub fn wrench(&mut self, tile_x: i32, tile_y: i32) {
        let netid = self.state.netid;
        self.input(&format!(
            "action|wrench\n|netid|{netid}\n|tileX|{tile_x}\n|tileY|{tile_y}"
        ));
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// One registered command.
pub struct Command {
    pub name: &'static str,
    pub summary: &'static str,
    pub keywords: &'static str,
    pub handler: fn(&mut Context, &str),
}

/// Every registered command, keyed by its lower-case name.
pub static REGISTRY: LazyLock<HashMap<&'static str, Command>> = LazyLock::new(|| {
        let mut map: HashMap<&'static str, Command> = HashMap::new();
        for cmd in crate::commands::ALL {
            map.insert(cmd.name, Command {
                name: cmd.name,
                summary: cmd.summary,
                keywords: cmd.keywords,
                handler: cmd.handler,
            });
        }
        map
    });

/// Alias to command name.
///
/// The reference build registers commands and aliases into one table, so one
/// alias can be claimed by several commands. The rules here make that
/// deterministic: a command's own name always wins over another command's
/// alias, and between two aliases the first registered wins.
pub static ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
        let mut map: HashMap<&'static str, &'static str> = HashMap::new();
        // Command names are reserved before any alias is considered.
        for cmd in crate::commands::ALL {
            map.insert(cmd.name, cmd.name);
        }
        for cmd in crate::commands::ALL {
            for alias in cmd.keywords.split(',').map(str::trim).filter(|a| !a.is_empty()) {
                map.entry(alias).or_insert(cmd.name);
            }
        }
        map
    });

/// Whether a chat line looks like a command.
pub fn is_command(text: &str) -> bool {
    let t = text.trim();
    t.starts_with('/') && {
        let rest = &t[1..];
        rest.chars()
            .next()
            .map(|c| c.is_ascii_alphanumeric() || c == '_')
            .unwrap_or(false)
    }
}

/// Split `/name args` into the name and its arguments.
pub fn split_command(text: &str) -> Option<(&str, &str)> {
    let t = text.trim();
    let rest = t.strip_prefix('/')?;
    let mut parts = rest.splitn(2, char::is_whitespace);
    let name = parts.next()?;
    let args = parts.next().unwrap_or("").trim();
    Some((name, args))
}

/// Handle a button press on the options page.
pub fn options_button(ctx: &mut Context, button: &str) -> bool {
    let Some(index) = button.strip_prefix("opt_").and_then(|n| n.parse::<usize>().ok()) else {
        return false;
    };
    crate::options::apply(ctx, index);
    true
}

/// Run a chat command. Returns true when the line was consumed.
pub fn dispatch(ctx: &mut Context, text: &str) -> bool {
    let Some((name, args)) = split_command(text) else {
        return false;
    };
    let lower = name.to_ascii_lowercase();
    let key = if REGISTRY.contains_key(lower.as_str()) {
        lower.as_str()
    } else {
        match ALIASES.get(lower.as_str()) {
            Some(c) => *c,
            None => {
                ctx.console(&format!(
                    "`4Unknown command /{lower}. Type /proxy for the list.``"
                ));
                return false;
            }
        }
    };

    let cmd = &REGISTRY[key];
    (cmd.handler)(ctx, args);
    true
}

/// Per-frame work: the automation tickers.
pub fn tick(ctx: &mut Context) {
    crate::automation::tick(ctx);
    crate::casino::tick(ctx);
}

/// Send a `GENERIC_TEXT` frame, used by the offline shell.
pub fn generic_text(text: &str) -> Vec<u8> {
    lzprotocol::text_packet(text, NetMessage::GenericText)
}
