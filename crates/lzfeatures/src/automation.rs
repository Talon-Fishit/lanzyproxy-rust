//! Automation: the farming and harvesting tickers, and their pages.

use lzprotocol::Variant;

use crate::common::{nearest_item_tile, scan_radius, toggle, walk_to};
use crate::Context;

const PROVIDER_ITEMS: [u32; 9] = [304, 302, 305, 306, 337, 339, 341, 1796, 2608];
const TREE_ITEMS: [u32; 9] = [2, 3, 4, 5, 6, 7, 8, 9, 10];

fn interval(ctx: &Context, key: &str) -> f64 {
    ctx.config
        .get("intervals")
        .get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or(150.0)
}

/// Whether a named timer has elapsed.
fn ready(ctx: &mut Context, key: &str, interval_ms: f64) -> bool {
    let now = lzstate::now();
    let last = ctx.state.timers.get(key).copied().unwrap_or(0.0);
    if now - last < interval_ms / 1000.0 {
        return false;
    }
    ctx.state.timers.insert(key.to_string(), now);
    true
}

fn page(ctx: &mut Context, key: &str, title: &str) {
    let mut d = ctx.page(&format!("lanzy_{key}"), &format!("`2{title}``"));
    d.textbox(&format!(
        "Current: {} | Interval: {}ms",
        if ctx.config.get_bool(key) { "`2ON``" } else { "`4OFF``" },
        interval(ctx, key) as i64
    ));
    d.button(&format!("{key}_toggle"), "Toggle");
    d.button(&format!("{key}_slow"), "Slower (-50ms)");
    d.button(&format!("{key}_fast"), "Faster (+50ms)");
    ctx.dialog(&format!("lanzy_{key}"), &d);
}

pub fn autofarm(ctx: &mut Context, args: &str) {
    match args.trim() {
        "on" => ctx.config.set_bool("autofarm", true),
        "off" => ctx.config.set_bool("autofarm", false),
        _ => {}
    }
    page(ctx, "autofarm", "Auto Farm");
}

pub fn farm(ctx: &mut Context, _args: &str) {
    toggle(ctx, "autofarm", "AutoFarm");
}

pub fn autoplant(ctx: &mut Context, _args: &str) {
    page(ctx, "autoplant", "Auto Plant");
}

pub fn plant(ctx: &mut Context, _args: &str) {
    toggle(ctx, "autoplant", "AutoPlant");
}

pub fn autoharvest(ctx: &mut Context, _args: &str) {
    page(ctx, "autoharvest", "Auto Harvest");
}

pub fn harvest(ctx: &mut Context, _args: &str) {
    toggle(ctx, "autoharvest", "AutoHarvest");
}

pub fn autocollect(ctx: &mut Context, _args: &str) {
    page(ctx, "autocollect", "Autocollect");
}

pub fn collect(ctx: &mut Context, _args: &str) {
    let Some(target) = nearest_item_tile(ctx, "") else {
        ctx.console("`4No floating items tracked in this world.``");
        return;
    };
    if walk_to(ctx, target) {
        ctx.punch(target.0, target.1);
        ctx.console(&format!("`2Collecting item near ({}, {})``", target.0, target.1));
    }
}

pub fn autocook(ctx: &mut Context, _args: &str) {
    page(ctx, "autocook", "Auto Cook");
}

pub fn startcook(ctx: &mut Context, _args: &str) {
    toggle(ctx, "autocook", "AutoCook");
    if ctx.config.get_bool("autocook") {
        ctx.dialog_return(&[("dialog_name", "oven".into()), ("button", "cook".into())]);
    }
}

pub fn fish(ctx: &mut Context, _args: &str) {
    toggle(ctx, "autofish", "AutoFish");
}

pub fn providers(ctx: &mut Context, _args: &str) {
    toggle(ctx, "providers", "Providers");
}

pub fn crime(ctx: &mut Context, _args: &str) {
    toggle(ctx, "autocrime", "AutoCrime");
}

pub fn acc(ctx: &mut Context, _args: &str) {
    toggle(ctx, "auto_accept", "AutoAcceptAccess");
}

pub fn autolock(ctx: &mut Context, _args: &str) {
    toggle(ctx, "autolock", "AutoWrenchLock");
}

pub fn auto(ctx: &mut Context, _args: &str) {
    let on_join = ctx.config.get_bool("auto_on_join");
    let words = ctx.config.get("autoban_words").as_array().map(|a| a.len()).unwrap_or(0);
    let players = ctx
        .config
        .get("autoban_players")
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let pulled = ctx
        .config
        .get("autopull_players")
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let mut d = ctx.page("lanzy_auto", "`2Auto Ban / Pull On Join``");
    d.toggle("auto_on_join", "Auto action when player joins", on_join);
    d.textbox(&format!("Banned words: {words}"));
    d.textbox(&format!("Banned players: {players}"));
    d.textbox(&format!("Pulled players: {pulled}"));
    d.button("auto_on_join", "Toggle auto action");
    d.button("auto_wipe", "Auto ban all on mod join");
    ctx.dialog("lanzy_auto", &d);
}

/// The anti-effect toggles: gravity, punch and lava immunity.
pub fn anti(ctx: &mut Context, _args: &str) {
    let mut d = ctx.page("lanzy_anti", "`2Anti Effects``");
    d.toggle("antigravity", "Antigravity", ctx.config.get_bool("antigravity"));
    d.toggle("antipunch", "Antipunch", ctx.config.get_bool("antipunch"));
    d.toggle("antilava", "Anti lava damage", ctx.config.get_bool("antilava"));
    d.toggle("frozen", "Anti snowball and slow", ctx.config.get_bool("frozen"));
    d.toggle("floating", "Anti balloon and floating", ctx.config.get_bool("floating"));
    d.textbox("Each effect toggles on its own.");
    ctx.dialog("lanzy_anti", &d);
}

/// Per-frame automation work.
pub fn tick(ctx: &mut Context) {
    if ctx.config.get_bool("autofarm") && ready(ctx, "autofarm", interval(ctx, "autofarm")) {
        let hits = scan_radius(ctx, 2);
        for (tx, ty, _) in hits.iter().take(6) {
            ctx.punch(*tx, *ty);
        }
    }

    if ctx.config.get_bool("autoplant") && ready(ctx, "autoplant", interval(ctx, "autoplant")) {
        for (tx, ty, tile) in scan_radius(ctx, 3) {
            if tile.empty() {
                ctx.place(tx, ty, 2, 0);
                break;
            }
        }
    }

    if ctx.config.get_bool("autoharvest")
        && ready(ctx, "autoharvest", interval(ctx, "autoharvest"))
    {
        for (tx, ty, tile) in scan_radius(ctx, 4) {
            if TREE_ITEMS.contains(&tile.fg) {
                ctx.punch(tx, ty);
            }
        }
    }

    if ctx.config.get_bool("autocollect")
        && ready(ctx, "autocollect", interval(ctx, "autocollect"))
    {
        if let Some(target) = nearest_item_tile(ctx, "") {
            walk_to(ctx, target);
            ctx.punch(target.0, target.1);
        }
    }

    if ctx.config.get_bool("providers") && ready(ctx, "providers", interval(ctx, "providers")) {
        for (tx, ty, tile) in scan_radius(ctx, 4) {
            if PROVIDER_ITEMS.contains(&tile.fg) {
                ctx.punch(tx, ty);
            }
        }
    }

    if ctx.config.get_bool("autofish") && ready(ctx, "autofish", interval(ctx, "autofish")) {
        ctx.input("action|fish");
    }

    if ctx.config.get_bool("autocook") && ready(ctx, "autocook", interval(ctx, "autocook")) {
        ctx.dialog_return(&[("dialog_name", "oven".into()), ("button", "cook".into())]);
    }

    if ctx.config.get_bool("autocrime") && ready(ctx, "autocrime", 900.0) {
        ctx.dialog_return(&[("dialog_name", "crime".into()), ("button", "solve".into())]);
    }

    let automsg = ctx.config.get_str("automsg");
    if !automsg.is_empty() {
        let delay = ctx.config.get_f64("automsg_delay") * 1000.0;
        if ready(ctx, "automsg", delay) {
            ctx.say(&automsg);
        }
    }
}

/// A `CALL_FUNCTION` variant, used by the offline shell.
pub fn call_function(name: &str, params: &[Variant]) -> Vec<u8> {
    let mut list = lzprotocol::VariantList::with_func(name);
    for p in params {
        list.add(p.clone());
    }
    lzprotocol::variant_packet(&list)
}
