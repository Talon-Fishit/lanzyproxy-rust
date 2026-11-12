//! World tools and the world-data decoder.

use lzstate::Tile;

use crate::common::scan_radius;
use crate::Context;

pub fn world(ctx: &mut Context, _args: &str) {
    let mut d = ctx.page("lanzy_world", "`2World Options``");
    d.textbox(&format!("World: {}", ctx.state.world_name));
    d.textbox(&format!(
        "Players: {} | Tiles: {}",
        ctx.state.players.len(),
        ctx.state.tiles.len()
    ));
    d.textbox(&format!(
        "World locks: {} | Gems: {}",
        ctx.state.world_locks, ctx.state.gems
    ));
    d.button("world_unban", "Unban world");
    d.button("world_pullall", "Pull all");
    d.button("world_kickall", "Kick all");
    ctx.dialog("lanzy_world", &d);
}

pub fn worlds(ctx: &mut Context, _args: &str) {
    let saved = ctx.config.get("saved_worlds");
    let names: Vec<String> = saved
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    if names.is_empty() {
        ctx.console("`9No saved worlds yet. Use /list to save one.``");
        return;
    }
    ctx.console(&format!("`2Worlds: {}``", names.join(", ")));
}

pub fn list_worlds(ctx: &mut Context, args: &str) {
    let mut saved = ctx.config.get("saved_worlds");
    if !saved.is_object() {
        saved = serde_json::Value::Object(serde_json::Map::new());
    }
    if !args.trim().is_empty() {
        let (name, door) = match args.trim().split_once(' ') {
            Some((n, d)) => (n.to_string(), d.to_string()),
            None => (args.trim().to_string(), String::new()),
        };
        let value = if door.is_empty() {
            ctx.state.world_name.clone()
        } else {
            door
        };
        saved.as_object_mut().unwrap().insert(name.clone(), serde_json::Value::String(value));
        ctx.config.set("saved_worlds", saved);
        ctx.console(&format!("`2Saved world {name}``"));
        return;
    }
    let mut d = ctx.page("lanzy_list", "`2Saved Worlds``");
    let names: Vec<String> = saved
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    if names.is_empty() {
        d.textbox("No saved worlds. Use /list <name> to add one.");
    }
    for name in names.iter().take(12) {
        d.button(&format!("warp_{name}"), name);
    }
    ctx.dialog("lanzy_list", &d);
}

pub fn scan(ctx: &mut Context, _args: &str) {
    let hits = scan_radius(ctx, 10);
    for (tx, ty, _) in hits.iter().take(20) {
        ctx.punch(*tx, *ty);
    }
    ctx.console(&format!(
        "`2Extract-O-Snap: punched {} tiles``",
        hits.len().min(20)
    ));
}

/// Decode `SEND_MAP_DATA` into tiles. Returns the number of tiles parsed.
///
/// The layout varies between game versions, so this is best effort: it reads
/// the common `uint16 version, uint32 tile_count` header followed by
/// `uint16 fg, uint16 bg, uint16 flags` records, and skips an optional extra
/// blob when the flags say one is present.
pub fn decode_world_data(ctx: &mut Context, payload: &[u8]) -> usize {
    ctx.state.last_map_raw = Some(payload.to_vec());
    if payload.len() < 8 {
        ctx.log("map data too short");
        return 0;
    }
    let tile_count = u32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]) as usize;
    if tile_count == 0 || tile_count > 30_000 {
        ctx.log(&format!("implausible tile count {tile_count}, skipping decode"));
        return 0;
    }

    let mut offset = 6usize;
    let mut parsed = 0usize;
    for _ in 0..tile_count {
        if offset + 6 > payload.len() {
            break;
        }
        let fg = u16::from_le_bytes([payload[offset], payload[offset + 1]]) as u32;
        let bg = u16::from_le_bytes([payload[offset + 2], payload[offset + 3]]) as u32;
        let flags = u16::from_le_bytes([payload[offset + 4], payload[offset + 5]]) as u32;
        offset += 6;

        if flags & 0x1 != 0 {
            if offset + 2 > payload.len() {
                break;
            }
            let extra_len =
                u16::from_le_bytes([payload[offset], payload[offset + 1]]) as usize;
            offset += 2 + extra_len;
            if offset > payload.len() {
                break;
            }
        }

        let width = if ctx.state.map_width > 0 {
            ctx.state.map_width
        } else {
            100
        };
        let x = (parsed % width.max(1) as usize) as i32;
        let y = (parsed / width.max(1) as usize) as i32;
        ctx.state.tiles.insert((x, y), Tile { fg, bg, flags });
        parsed += 1;
    }

    if parsed > 0 {
        ctx.state.map_width = ctx.state.map_width.max(100);
        ctx.state.map_height = ctx.state.map_height.max((parsed / 100 + 1) as i32);
        ctx.log(&format!("decoded {parsed} tiles"));
    }
    parsed
}
