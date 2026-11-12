//! Visual spoofing: clothes, titles, weather, country, UI colours, movement.

use serde_json::Value;

use crate::common::toggle;
use crate::Context;

const CLOTHING_SETS: [(&str, &[i64]); 4] = [
    ("rayman", &[1296, 4308, 4318, 1264]),
    ("legend", &[1050, 1052, 1054, 1056]),
    ("angel", &[44, 45, 46, 47]),
    ("naked", &[]),
];

fn apply_visuals(ctx: &mut Context) {
    let clothes = ctx.config.get("visual_clothes");
    let joined = clothes
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    ctx.console(&format!("`2Visual clothes set: [{}]``", if joined.is_empty() { "none" } else { &joined }));
    ctx.log(&format!("visual clothes -> {joined}"));
}

pub fn clothes(ctx: &mut Context, args: &str) {
    let mut d = ctx.page("lanzy_clothes", "`2Visual Clothes``");
    let active = ctx.config.get("visual_clothes");
    let active_text = active
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "none".into());
    d.textbox(&format!("Active: {active_text}"));
    for (name, _) in CLOTHING_SETS {
        d.button(&format!("set_{name}"), &capitalise(name));
    }
    d.button("clothes_off", "Remove visual clothes");
    d.textbox("Use /clothes <id,id,id> for a custom set.");
    if !args.trim().is_empty() {
        let ids: Option<Vec<i64>> = args
            .replace(' ', "")
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<i64>().ok())
            .collect();
        match ids {
            Some(ids) => {
                ctx.config.set(
                    "visual_clothes",
                    Value::Array(ids.into_iter().map(Value::from).collect()),
                );
                apply_visuals(ctx);
            }
            None => {
                ctx.console("`4Invalid item id list.``");
                return;
            }
        }
        return;
    }
    ctx.dialog("lanzy_clothes", &d);
}

fn capitalise(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub fn unequip(ctx: &mut Context, _args: &str) {
    ctx.config.set("visual_clothes", Value::Array(vec![]));
    ctx.console("`2Visual clothes removed.``");
}

pub fn titles(ctx: &mut Context, args: &str) {
    let mut d = ctx.page("lanzy_titles", "`2Visual Titles``");
    let current = ctx.config.get_str("visual_title");
    d.textbox(&format!(
        "Current: {}",
        if current.is_empty() { "none" } else { &current }
    ));
    for title in ["Legend", "G4G", "Doctor", "Max Level"] {
        d.button(
            &format!("title_{}", title.to_lowercase().replace(' ', "_")),
            title,
        );
    }
    d.button("title_clear", "Clear title");
    if !args.trim().is_empty() {
        ctx.config.set_str("visual_title", args.trim());
        ctx.console(&format!("`2Visual title: {}``", args.trim()));
        return;
    }
    ctx.dialog("lanzy_titles", &d);
}

pub fn weather(ctx: &mut Context, args: &str) {
    let mut d = ctx.page("lanzy_weather", "`2Visual Weather``");
    let current = ctx.config.get_str("weather");
    d.textbox(&format!(
        "Current: {}",
        if current.is_empty() { "default" } else { &current }
    ));
    for name in ["Sunny", "Rainy", "Snowy", "Night", "Aurora"] {
        d.button(&format!("weather_{}", name.to_lowercase()), name);
    }
    if !args.trim().is_empty() {
        let title = capitalise(args.trim());
        ctx.config.set_str("weather", &title);
        ctx.variant(
            "OnSetCurrentWeather",
            &[lzprotocol::Variant::String(title.clone())],
        );
        ctx.console(&format!("`2Weather -> {title}``"));
        return;
    }
    ctx.dialog("lanzy_weather", &d);
}

pub fn country(ctx: &mut Context, args: &str) {
    let code: String = args.trim().to_uppercase().chars().take(2).collect();
    if code.is_empty() {
        ctx.console("`9Usage: /country <2-letter code> (e.g. ID, US, PH)``");
        return;
    }
    ctx.config.set_str("country", &code);
    ctx.input(&format!("action|setCountry\n|code|{code}"));
    ctx.console(&format!("`2Country flag -> {code}``"));
}

pub fn spirit(ctx: &mut Context, _args: &str) {
    let mut d = ctx.page("lanzy_spirit", "`2Spirit Board``");
    d.textbox(&format!("World: {}", ctx.state.world_name));
    d.textbox(&format!("Nearby players: {}", ctx.state.players.len()));
    d.textbox(&format!("Tracked mods: {}", ctx.state.mods.len()));
    ctx.dialog("lanzy_spirit", &d);
}

pub fn ui(ctx: &mut Context, args: &str) {
    let mut d = ctx.page("lanzy_ui", "`2UI Colors``");
    let current = ctx.config.get_str("ui_color");
    d.textbox(&format!(
        "Current accent: {}",
        if current.is_empty() { "default" } else { &current }
    ));
    for (name, code) in [
        ("Green", "`2"),
        ("Red", "`4"),
        ("Blue", "`9"),
        ("Purple", "`#"),
        ("White", "`w"),
    ] {
        d.button(&format!("ui_{}", name.to_lowercase()), &format!("{code}{name}``"));
    }
    if !args.trim().is_empty() {
        ctx.config.set_str("ui_color", args.trim());
        ctx.console(&format!("UI accent -> {}", args.trim()));
        return;
    }
    ctx.dialog("lanzy_ui", &d);
}

pub fn speed(ctx: &mut Context, args: &str) {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if !parts.is_empty() {
        let mut failed = false;
        if parts.len() == 1 {
            match parts[0].parse::<f64>() {
                Ok(v) => ctx.config.set("speed", Value::from(v)),
                Err(_) => {
                    ctx.console("`4Usage: /speed 1.5  |  /speed jump=2 gravity=0.5``");
                    return;
                }
            }
        } else {
            for part in parts {
                let Some((key, value)) = part.split_once('=') else {
                    failed = true;
                    continue;
                };
                let Ok(v) = value.parse::<f64>() else {
                    failed = true;
                    continue;
                };
                let key = match key {
                    "fall" => "fall_speed",
                    "water" => "water_speed",
                    "speed" | "jump" | "gravity" => key,
                    _ => {
                        failed = true;
                        continue;
                    }
                };
                ctx.config.set(key, Value::from(v));
            }
        }
        if failed {
            ctx.console("`4Usage: /speed 1.5  |  /speed jump=2 gravity=0.5``");
            return;
        }
        ctx.console(&format!(
            "`2speed={} jump={} gravity={}``",
            ctx.config.get("speed"),
            ctx.config.get("jump"),
            ctx.config.get("gravity")
        ));
        return;
    }
    let mut d = ctx.page("lanzy_speed", "`2Custom Player Speed``");
    d.textbox(&format!(
        "Speed: {} | Jump: {}",
        ctx.config.get("speed"),
        ctx.config.get("jump")
    ));
    d.textbox(&format!(
        "Gravity: {} | Fall: {}",
        ctx.config.get("gravity"),
        ctx.config.get("fall_speed")
    ));
    d.textbox(&format!("Water: {}", ctx.config.get("water_speed")));
    d.toggle(
        "unlimited_jump",
        "Unlimited jump",
        ctx.config.get_bool("unlimited_jump"),
    );
    d.toggle(
        "ghost_mode",
        "Ghost / mod move mode",
        ctx.config.get_bool("ghost_mode"),
    );
    d.textbox("Edit: /speed 1.5 | /speed jump=2 gravity=0.5");
    ctx.dialog("lanzy_speed", &d);
}

/// The clothing presets, exposed so the settings page can list them.
pub fn clothing_sets() -> &'static [(&'static str, &'static [i64])] {
    &CLOTHING_SETS
}

/// Toggle helper kept for symmetry with the Python feature modules.
pub fn toggle_setting(ctx: &mut Context, key: &str, label: &str) {
    toggle(ctx, key, label);
}
