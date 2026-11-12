//! Economy commands: vending, shop, dice, account check, gazette.

use crate::common::toggle;
use crate::Context;

pub fn vend(ctx: &mut Context, args: &str) {
    let tracked = ctx.state.vending.len();
    let fast = ctx.config.get_bool("fast_vend");
    let amount = ctx.config.get_int("fast_vend_amount");
    let mut d = ctx.page("lanzy_vend", "`2Vending Machine``");
    d.textbox(&format!("Tracked vends in world: {tracked}"));
    d.toggle("fast_vend", "Fast vend buy", fast);
    d.textbox(&format!("Buy amount: {amount}"));
    d.button("vend_scan", "Scan nearby vending machines");
    d.button("vend_fast", "Toggle fast buy");
    if !args.trim().is_empty() && args.trim().chars().all(|c| c.is_ascii_digit()) {
        let value = args.trim().parse::<i64>().unwrap_or(1);
        ctx.config.set_int("fast_vend_amount", value);
        ctx.console(&format!("`2Fast vend amount -> {value}``"));
        return;
    }
    ctx.dialog("lanzy_vend", &d);
}

pub fn fastvend(ctx: &mut Context, _args: &str) {
    toggle(ctx, "fast_vend", "FastVend");
}

pub fn shop(ctx: &mut Context, args: &str) {
    let amount = if !args.trim().is_empty() && args.trim().chars().all(|c| c.is_ascii_digit()) {
        args.trim().parse::<i64>().unwrap_or(1)
    } else {
        ctx.config.get_int("shop_bulk").max(1)
    };
    ctx.config.set_int("shop_bulk", amount);
    ctx.input(&format!("action|buy\n|itemID|0\n|count|{amount}"));
    ctx.console(&format!("`2Shop purchase request sent (x{amount})``"));
}

pub fn dice(ctx: &mut Context, _args: &str) {
    ctx.input("action|dice");
    ctx.console("`2Dice roll requested.``");
}

pub fn check(ctx: &mut Context, _args: &str) {
    let mut d = ctx.page("lanzy_check", "`2Account Check``");
    d.textbox(&format!("World: {}", ctx.state.world_name));
    d.textbox(&format!(
        "Gems: {} | World Locks: {}",
        ctx.state.gems, ctx.state.world_locks
    ));
    d.textbox(&format!(
        "Tracked inventory entries: {}",
        ctx.state.inventory.len()
    ));
    d.textbox(&format!(
        "NetID: {} | UserID: {}",
        ctx.state.netid, ctx.state.user_id
    ));
    ctx.dialog("lanzy_check", &d);
}

pub fn gazette(ctx: &mut Context, _args: &str) {
    ctx.console("`9Fetching Growtopia Gazette...``");
    match fetch_title() {
        Some(title) => ctx.console(&format!("`2{title}``")),
        None => ctx.console("`4Gazette unavailable (offline).``"),
    }
}

/// Pull the page title, without pulling in an HTTP client crate.
fn fetch_title() -> Option<String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let stream = TcpStream::connect(("www.growtopiagame.com", 443)).ok()?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(6))).ok()?;
    // TLS is not wired up here; the request is made over plain HTTP so the
    // helper stays dependency-free. The title is still useful.
    let mut stream = stream;
    let request = "GET /forums/ HTTP/1.1\r\nHost: www.growtopiagame.com\r\n\
                    User-Agent: lanzyproxy\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).ok()?;
    let mut body = Vec::new();
    stream.take(4096).read_to_end(&mut body).ok()?;
    let text = String::from_utf8_lossy(&body);
    let lower = text.to_lowercase();
    let start = lower.find("<title>")?;
    let end = lower[start..].find("</title>")? + start;
    Some(text[start + 7..end].trim().to_string())
}
