//! Navigation: pathfinding, world scanning, the offline renderer.

use std::path::PathBuf;

use crate::common::{tile_color, walk_to};
use crate::Context;

pub fn pathfind(ctx: &mut Context, args: &str) {
    if !args.trim().is_empty() && args.trim().chars().all(|c| c.is_ascii_digit()) {
        let value = args.trim().parse::<i64>().unwrap_or(80);
        ctx.config.set_int("pathfind_delay", value);
        ctx.console(&format!("`2Pathfind delay -> {value}ms``"));
        return;
    }
    let mut d = ctx.page("lanzy_path", "`2Pathfinder Settings``");
    d.textbox(&format!(
        "Step delay: {}ms",
        ctx.config.get_int("pathfind_delay")
    ));
    d.textbox(&format!(
        "Speed multiplier: {}",
        ctx.config.get("pathfind_speed")
    ));
    ctx.dialog("lanzy_path", &d);
}

pub fn pfind(ctx: &mut Context, _args: &str) {
    let Some(target) = crate::common::nearest_item_tile(ctx, "") else {
        ctx.console("`4No tracked item found.``");
        return;
    };
    ctx.punch(target.0, target.1);
    ctx.console(&format!("`2Pathfound to {:?}``", target));
}

pub fn pvend(ctx: &mut Context, _args: &str) {
    if ctx.state.vending.is_empty() {
        ctx.console("`4No vending data captured. Open the world first.``");
        return;
    }
    // A vending entry is a [name, price] pair.
    let best = ctx
        .state
        .vending
        .iter()
        .min_by_key(|v| v.get(1).and_then(|p| p.as_i64()).unwrap_or(i64::MAX));
    if let Some(best) = best {
        let item = best
            .first()
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let price = best
            .get(1)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "?".into());
        ctx.console(&format!("`2Vending target: {item} price {price}``"));
    }
}

pub fn find(ctx: &mut Context, args: &str) {
    let needle = args.trim().to_lowercase();
    if needle.is_empty() {
        ctx.console("`9Usage: /find <item name or id>``");
        return;
    }
    let hits: Vec<(u32, f32, f32)> = ctx
        .state
        .spawn_items
        .iter()
        .filter(|item| item.item_id.to_string().contains(&needle))
        .map(|item| (item.item_id, item.x, item.y))
        .collect();
    ctx.console(&format!("`2Matches in tracked drops: {}``", hits.len()));
    for (item_id, x, y) in hits.iter().take(12) {
        ctx.console(&format!("  id={item_id} x={x:.0} y={y:.0}"));
    }
}

pub fn render(ctx: &mut Context, _args: &str) {
    if ctx.state.tiles.is_empty() {
        ctx.console("`4No map data captured yet.``");
        return;
    }
    let width = ctx.state.map_width.max(1) as usize;
    let height = ctx.state.map_height.max(1) as usize;
    let pixels: Vec<Vec<(u8, u8, u8)>> = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| tile_color(ctx.state.tile(x as i32, y as i32), x as i32, y as i32))
                .collect()
        })
        .collect();

    let dir = render_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path: PathBuf = dir.join(format!(
        "{}_{}.png",
        ctx.state.world_name,
        lzstate::now() as i64
    ));
    if crate::common::write_png(&path, width, height, &pixels) {
        ctx.console(&format!("`2Render saved: {}``", path.display()));
    } else {
        ctx.console("`4Render failed to write file.``");
    }
}

fn render_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("renders")))
        .unwrap_or_else(|| PathBuf::from("renders"))
}

pub fn parkour(ctx: &mut Context, _args: &str) {
    let goal = ctx
        .state
        .tiles
        .iter()
        .filter(|((_, y), tile)| tile.fg == 0 && *y > 0)
        .map(|((x, y), _)| (*x, *y))
        .last();
    let Some(goal) = goal else {
        ctx.console("`4No navigation goal found in map.``");
        return;
    };
    if walk_to(ctx, goal) {
        ctx.console(&format!("`2Pathfinding to goal {goal:?}``"));
    }
}

pub fn blocks(ctx: &mut Context, _args: &str) {
    growscan(ctx, "blocks");
}

pub fn gs(ctx: &mut Context, _args: &str) {
    growscan(ctx, "all");
}

pub fn items(ctx: &mut Context, _args: &str) {
    growscan(ctx, "items");
}

pub fn tree(ctx: &mut Context, _args: &str) {
    let fruits: Vec<((i32, i32), u32)> = ctx
        .state
        .tiles
        .iter()
        .filter(|(_, t)| (2..=8).contains(&t.fg))
        .map(|((x, y), t)| ((*x, *y), t.fg))
        .collect();
    ctx.console(&format!("`2Plant tiles tracked: {}``", fruits.len()));
    for ((x, y), fg) in fruits.iter().take(12) {
        ctx.console(&format!("  ({x},{y}) item={fg}"));
    }
}

pub fn chest(ctx: &mut Context, _args: &str) {
    if ctx.state.chests.is_empty() {
        ctx.console("`4No chest data captured.``");
        return;
    }
    let entries: Vec<((i32, i32), usize)> = ctx
        .state
        .chests
        .iter()
        .map(|((x, y), v)| ((*x, *y), v.len()))
        .collect();
    for ((x, y), count) in entries.iter().take(10) {
        ctx.console(&format!("`2Chest ({x},{y}): {count} entries``"));
    }
}

fn growscan(ctx: &mut Context, mode: &str) {
    let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for tile in ctx.state.tiles.values() {
        if !tile.empty() {
            *counts.entry(tile.fg).or_insert(0) += 1;
        }
    }
    let mut sorted: Vec<(u32, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let mut d = ctx.page("lanzy_growscan", "`2Growscan 9000``");
    d.textbox(&format!(
        "World: {} | tiles: {}",
        ctx.state.world_name,
        ctx.state.tiles.len()
    ));
    if mode == "all" || mode == "blocks" {
        for (item_id, amount) in sorted.iter().take(8) {
            d.textbox(&format!("item {item_id}: {amount}"));
        }
    }
    if mode == "all" || mode == "items" {
        for item in ctx.state.spawn_items.iter().take(8) {
            d.textbox(&format!(
                "drop {} @ {:.0},{:.0}",
                item.item_id, item.x, item.y
            ));
        }
    }
    ctx.dialog("lanzy_growscan", &d);
}
