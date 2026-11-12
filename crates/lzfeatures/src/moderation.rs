//! Moderation and player-tracking commands.

use crate::common::toggle;
use crate::Context;

const MOD_FLAGS: [&str; 3] = ["mod", "developer", "administrator"];

pub fn wrench(ctx: &mut Context, args: &str) {
    let mode = args.trim().to_lowercase();
    if ["pull", "kick", "ban"].contains(&mode.as_str()) {
        ctx.config.set_str("wrench_mode", &mode);
        ctx.console(&format!("`2Wrench mode -> {mode}``"));
        return;
    }
    let current = ctx.config.get_str("wrench_mode");
    ctx.console(&format!("`9Wrench mode: {current}``"));
    ctx.console("`9Use /wrench pull|kick|ban``");
}

pub fn locked(ctx: &mut Context, _args: &str) {
    if ctx.state.lock_owners.is_empty() {
        ctx.console("`9No lock data captured yet.``");
        return;
    }
    ctx.console(&format!(
        "`2Locked tiles tracked: {}``",
        ctx.state.lock_owners.len()
    ));
    let locks: Vec<((i32, i32), String)> = ctx
        .state
        .lock_owners
        .iter()
        .take(10)
        .map(|((x, y), owner)| ((*x, *y), owner.clone()))
        .collect();
    for ((x, y), owner) in locks {
        ctx.console(&format!("  ({x},{y}) -> {owner}"));
    }
}

pub fn lost(ctx: &mut Context, _args: &str) {
    let (tx, ty) = (ctx.state.tile_x, ctx.state.tile_y);
    ctx.wrench(tx, ty);
    ctx.dialog_return(&[("dialog_name", "lock_edit".into()), ("button", "public".into())]);
    ctx.console("`2Lock applied (public) to current tile.``");
}

pub fn moddetect(ctx: &mut Context, _args: &str) {
    toggle(ctx, "moddetect", "ModDetect");
}

pub fn track(ctx: &mut Context, args: &str) {
    match args.trim() {
        "on" => ctx.config.set_bool("track", true),
        "off" => ctx.config.set_bool("track", false),
        _ => {}
    }
    let enabled = ctx.config.get_bool("track");
    ctx.console(&format!(
        "Track dropped/collected items: {}",
        if enabled { "`2ON``" } else { "`4OFF``" }
    ));
}

pub fn pkickall(ctx: &mut Context, _args: &str) {
    let count = ctx.state.players.len();
    ctx.console(&format!("`2Kick all: {count} players in world.``"));
    let names: Vec<String> = ctx
        .state
        .players
        .values()
        .map(|p| p.name.clone())
        .collect();
    for name in names {
        ctx.input(&format!("action|kick\n|name|{name}"));
    }
}

pub fn hidden(ctx: &mut Context, _args: &str) {
    let hidden: Vec<String> = ctx
        .state
        .players
        .values()
        .filter(|p| p.invisible)
        .map(|p| p.name.clone())
        .collect();
    if hidden.is_empty() {
        ctx.console("`2No hidden moderators detected.``");
        return;
    }
    ctx.console(&format!("`2Hidden moderators: {}``", hidden.join(", ")));
}

/// The flags a player name can carry to mark a moderator.
pub fn mod_flags() -> &'static [&'static str] {
    &MOD_FLAGS
}
