//! Runtime state.
//!
//! Everything the proxy learns about the session while it is running: which
//! world, which tiles, who is in it, what has been picked up or dropped.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A world tile.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tile {
    pub fg: u32,
    pub bg: u32,
    pub flags: u32,
}

impl Tile {
    /// A tile with no foreground object.
    pub fn empty(&self) -> bool {
        self.fg == 0
    }
}

/// An item seen on the ground.
#[derive(Debug, Clone)]
pub struct TrackedItem {
    pub item_id: u32,
    pub amount: i32,
    pub x: f32,
    pub y: f32,
    pub netid: i32,
    pub source: String,
    pub seen: f64,
}

impl TrackedItem {
    /// A newly observed item.
    pub fn new(item_id: u32, amount: i32, x: f32, y: f32) -> Self {
        TrackedItem {
            item_id,
            amount,
            x,
            y,
            netid: 0,
            source: "drop".into(),
            seen: now(),
        }
    }
}

/// A player seen in the world.
#[derive(Debug, Clone, Default)]
pub struct PlayerInfo {
    pub netid: i32,
    pub name: String,
    pub is_mod: bool,
    pub invisible: bool,
    pub x: f32,
    pub y: f32,
}

/// Seconds since the Unix epoch.
pub fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Everything the proxy tracks during a session.
#[derive(Debug, Clone)]
pub struct State {
    pub world: String,
    pub world_name: String,
    pub netid: i32,
    pub user_id: i32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub map_width: i32,
    pub map_height: i32,
    pub tiles: HashMap<(i32, i32), Tile>,
    pub spawn_items: Vec<TrackedItem>,
    pub players: HashMap<i32, PlayerInfo>,
    pub mods: HashMap<String, f64>,
    pub vending: Vec<Vec<serde_json::Value>>,
    pub chests: HashMap<(i32, i32), Vec<Vec<serde_json::Value>>>,
    pub lock_owners: HashMap<(i32, i32), String>,
    pub inventory: Vec<(u32, i32)>,
    pub gems: i32,
    pub world_locks: i32,
    pub connected: bool,
    pub authenticated: bool,
    pub last_state: Option<Vec<u8>>,
    pub last_map_raw: Option<Vec<u8>>,
    pub timers: HashMap<String, f64>,
    pub logs: Vec<(f64, String)>,
    pub saved_positions: HashMap<String, (i32, i32)>,
    pub spam_mode: i32,
    /// The last measured ping round trip, in milliseconds.
    pub last_ping_ms: i64,
    /// How many public entrances or gates are open.
    pub gates_open: i32,
    /// Spins announced by the wheel that have not been matched to a claim yet.
    pub pending_spins: Vec<crate::PendingSpin>,
}

/// A wheel spin waiting to be matched against a player claim.
#[derive(Debug, Clone, Default)]
pub struct PendingSpin {
    pub name: String,
    pub number: u32,
    pub seen: f64,
}


impl Default for State {
    fn default() -> Self {
        State {
            world: "EXIT".into(),
            world_name: "EXIT".into(),
            netid: 0,
            user_id: 0,
            pos_x: 0.0,
            pos_y: 0.0,
            tile_x: 0,
            tile_y: 0,
            map_width: 0,
            map_height: 0,
            tiles: HashMap::new(),
            spawn_items: Vec::new(),
            players: HashMap::new(),
            mods: HashMap::new(),
            vending: Vec::new(),
            chests: HashMap::new(),
            lock_owners: HashMap::new(),
            inventory: Vec::new(),
            gems: 0,
            world_locks: 0,
            connected: false,
            authenticated: false,
            last_state: None,
            last_map_raw: None,
            timers: HashMap::new(),
            logs: Vec::new(),
            saved_positions: HashMap::new(),
            spam_mode: 0,
            last_ping_ms: 0,
            gates_open: 0,
            pending_spins: Vec::new(),
        }
    }
}

impl State {
    /// A fresh session.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a line to the session log.
    pub fn log(&mut self, message: &str) {
        self.logs.push((now(), message.to_string()));
        if self.logs.len() > 500 {
            let excess = self.logs.len() - 500;
            self.logs.drain(0..excess);
        }
    }

    /// The tile at a position, if it is known.
    pub fn tile(&self, x: i32, y: i32) -> Option<&Tile> {
        self.tiles.get(&(x, y))
    }

    /// Whether the player currently stands on a tile with a sign or lock.
    pub fn is_tile_locked(&self) -> bool {
        self.tile(self.tile_x, self.tile_y)
            .map(|t| t.flags != 0)
            .unwrap_or(false)
    }

    /// Reset the per-session state, keeping the configuration-facing bits.
    pub fn reset_session(&mut self) {
        self.world = "EXIT".into();
        self.world_name = "EXIT".into();
        self.netid = 0;
        self.pos_x = 0.0;
        self.pos_y = 0.0;
        self.tile_x = 0;
        self.tile_y = 0;
        self.map_width = 0;
        self.map_height = 0;
        self.tiles.clear();
        self.spawn_items.clear();
        self.players.clear();
        self.mods.clear();
        self.vending.clear();
        self.chests.clear();
        self.lock_owners.clear();
        self.inventory.clear();
        self.gems = 0;
        self.world_locks = 0;
        self.connected = false;
        self.authenticated = false;
        self.last_state = None;
        self.last_map_raw = None;
        self.pending_spins.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_is_bounded() {
        let mut s = State::new();
        for i in 0..600 {
            s.log(&format!("line {i}"));
        }
        assert_eq!(s.logs.len(), 500);
    }

    #[test]
    fn empty_tile_check() {
        let t = Tile::default();
        assert!(t.empty());
    }
}
