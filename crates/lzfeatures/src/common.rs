//! Shared helpers: BFS pathfinding, tile scans, PNG output.

use std::collections::HashMap;
use std::collections::VecDeque;

use lzstate::Tile;

use crate::Context;

/// Grid BFS over empty foreground tiles. Returns the tiles to visit, excluding
/// the start.
pub fn bfs_path(
    ctx: &Context,
    start: (i32, i32),
    goal: (i32, i32),
    limit: usize,
) -> Vec<(i32, i32)> {
    if start == goal {
        return Vec::new();
    }
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    let mut came: HashMap<(i32, i32), Option<(i32, i32)>> = HashMap::new();
    queue.push_back(start);
    came.insert(start, None);

    while let Some(current) = queue.pop_front() {
        if came.len() >= limit {
            break;
        }
        let (cx, cy) = current;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = (cx + dx, cy + dy);
            if came.contains_key(&next) {
                continue;
            }
            let reachable = next == goal || walkable(ctx, next);
            if !reachable {
                continue;
            }
            came.insert(next, Some(current));
            if next == goal {
                let mut path = vec![next];
                while let Some(parent) = came.get(path.last().unwrap()) {
                    match parent {
                        Some(p) => path.push(*p),
                        None => break,
                    }
                }
                path.pop();
                path.reverse();
                return path;
            }
            queue.push_back(next);
        }
    }
    Vec::new()
}

/// Whether a tile can be walked onto.
pub fn walkable(ctx: &Context, pos: (i32, i32)) -> bool {
    match ctx.state.tile(pos.0, pos.1) {
        Some(tile) => tile.empty(),
        None => false,
    }
}

/// The tile of a tracked drop nearest the player.
pub fn nearest_item_tile(ctx: &Context, item_filter: &str) -> Option<(i32, i32)> {
    let mut best: Option<((i32, i32), f32)> = None;
    for item in &ctx.state.spawn_items {
        if !item_filter.is_empty() && !item.item_id.to_string().contains(item_filter) {
            continue;
        }
        let tx = (item.x / 32.0) as i32;
        let ty = (item.y / 32.0) as i32;
        let d = ((tx - ctx.state.tile_x).abs() + (ty - ctx.state.tile_y).abs()) as f32;
        if best.map(|(_, bd)| d < bd).unwrap_or(true) {
            best = Some(((tx, ty), d));
        }
    }
    best.map(|(pos, _)| pos)
}

/// Walk to a tile. Returns false when there is no path.
pub fn walk_to(ctx: &mut Context, goal: (i32, i32)) -> bool {
    let path = bfs_path(
        ctx,
        (ctx.state.tile_x, ctx.state.tile_y),
        goal,
        4000,
    );
    if path.is_empty() {
        let msg = format!("no path to {goal:?}");
        ctx.log(&msg);
        return false;
    }
    for (tx, ty) in path.iter().take(64) {
        let x = (*tx as f32) * 32.0 + 16.0;
        let y = (*ty as f32) * 32.0 + 16.0;
        ctx.move_to(x, y);
        ctx.state.pos_x = x;
        ctx.state.pos_y = y;
        ctx.state.tile_x = *tx;
        ctx.state.tile_y = *ty;
    }
    true
}

/// Every non-empty tile within `radius` of the player.
pub fn scan_radius(ctx: &Context, radius: i32) -> Vec<(i32, i32, Tile)> {
    let mut result = Vec::new();
    let bx = ctx.state.tile_x;
    let by = ctx.state.tile_y;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if let Some(tile) = ctx.state.tile(bx + dx, by + dy) {
                if !tile.empty() {
                    result.push((bx + dx, by + dy, *tile));
                }
            }
        }
    }
    result
}

/// Flip a boolean setting and report the new value.
pub fn toggle(ctx: &mut Context, key: &str, label: &str) {
    let value = !ctx.config.get_bool(key);
    ctx.config.set_bool(key, value);
    let text = if value { "`2ON``" } else { "`4OFF``" };
    ctx.console(&format!("{label}: {text}"));
    ctx.log(&format!("{key} -> {value}"));
}

/// The palette used for the offline world render.
pub const TILE_COLORS: [(u8, u8, u8); 8] = [
    (34, 139, 34),
    (0, 100, 0),
    (139, 69, 19),
    (128, 128, 128),
    (105, 105, 105),
    (0, 0, 139),
    (139, 0, 139),
    (184, 134, 11),
];

/// The colour a tile renders as.
pub fn tile_color(tile: Option<&Tile>, x: i32, y: i32) -> (u8, u8, u8) {
    match tile {
        None => (12, 12, 12),
        Some(t) if t.empty() => (18, 18, 28),
        Some(t) => {
            let idx = (t.fg as usize + (x as usize) * 3 + (y as usize) * 7) % TILE_COLORS.len();
            TILE_COLORS[idx]
        }
    }
}

/// Write a PNG (RGB8) without any image dependency.
pub fn write_png(
    path: &std::path::Path,
    width: usize,
    height: usize,
    pixels: &[Vec<(u8, u8, u8)>],
) -> bool {
    if width == 0 || height == 0 || pixels.len() < height {
        return false;
    }
    let mut raw: Vec<u8> = Vec::with_capacity(height * (1 + width * 3));
    for row in pixels.iter().take(height) {
        raw.push(0); // filter type: none
        for (r, g, b) in row {
            raw.extend_from_slice(&[*r, *g, *b]);
        }
    }

    fn chunk(tag: &[u8], body: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(body.len() + 12);
        out.extend_from_slice(&(body.len() as u32).to_be_bytes());
        out.extend_from_slice(tag);
        out.extend_from_slice(body);
        let mut hasher = crc32::Crc32::new();
        hasher.update(tag);
        hasher.update(body);
        out.extend_from_slice(&hasher.finalize().to_be_bytes());
        out
    }

    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);

    let mut png = Vec::new();
    png.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
    png.extend_from_slice(&chunk(b"IHDR", &header));
    png.extend_from_slice(&chunk(b"IDAT", &deflate(&raw)));
    png.extend_from_slice(&chunk(b"IEND", &[]));

    std::fs::write(path, png).is_ok()
}

/// A small zlib-deflate (stored blocks) compressor: PNG only needs a valid
/// zlib stream, and stored blocks keep this dependency-free.
fn deflate(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / 65535 * 5 + 16);
    // zlib header: deflate, 32K window, default level, no dictionary.
    out.extend_from_slice(&[0x78, 0x01]);
    let mut offset = 0;
    if data.is_empty() {
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xff, 0xff]);
    }
    while offset < data.len() {
        let take = (data.len() - offset).min(65535);
        let last = offset + take >= data.len();
        out.push(if last { 1 } else { 0 });
        out.extend_from_slice(&(take as u16).to_le_bytes());
        out.extend_from_slice(&(!(take as u16)).to_le_bytes());
        out.extend_from_slice(&data[offset..offset + take]);
        offset += take;
    }
    // Adler-32 checksum.
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    out.extend_from_slice(&((b << 16) | a).to_be_bytes());
    out
}

/// A minimal CRC-32, needed for PNG chunk checksums.
mod crc32 {
    pub struct Crc32(u32);

    impl Crc32 {
        pub fn new() -> Self {
            Crc32(0xFFFF_FFFF)
        }

        pub fn update(&mut self, bytes: &[u8]) {
            for &byte in bytes {
                self.0 ^= byte as u32;
                for _ in 0..8 {
                    let mask = if self.0 & 1 != 0 { 0xEDB8_8320 } else { 0 };
                    self.0 = (self.0 >> 1) ^ mask;
                }
            }
        }

        pub fn finalize(&self) -> u32 {
            self.0 ^ 0xFFFF_FFFF
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_starts_with_the_magic() {
        let pixels = vec![vec![(1u8, 2, 3); 2]; 2];
        let dir = std::env::temp_dir();
        let path = dir.join("lzfeatures-test.png");
        assert!(write_png(&path, 2, 2, &pixels));
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[..8], &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
        let _ = std::fs::remove_file(&path);
    }
}
