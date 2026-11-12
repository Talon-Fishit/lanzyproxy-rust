//! Casino / roulette.
//!
//! The wire strings below were recovered from a shipped Android proxy build,
//! not guessed:
//!
//! ```text
//! "spun the wheel and got "   the wheel broadcast
//! "`0[`2REAL```0]```` "       the REAL prefix
//! " `0[`4FAKE```0]```` "      the FAKE suffix
//! " `^REME: `#"               the REME suffix
//! " `^QEME: `#"               the QEME suffix
//! " `^LEME: `#"               the LEME suffix
//! ```
//!
//! A broadcast carrying a valid spin fragment is REAL; a claim without one is
//! FAKE. Two signals count as a fragment: the wheel's colour-coded player
//! prefix and a spin verified inside the verification window.

use std::collections::HashMap;

use serde_json::Value;

use crate::Context;

/// The wheel broadcast phrase.
pub const SPIN_PHRASE: &str = "spun the wheel and got";
/// The REAL prefix.
pub const TAG_REAL: &str = "`0[`2REAL```0]```` ";
/// The FAKE suffix.
pub const TAG_FAKE: &str = " `0[`4FAKE```0]```` ";
/// The HIDDEN prefix.
pub const HIDDEN_PREFIX: &str = "```b[`4HIDDEN`b]`` `0spun the wheel and got ";
/// The REME suffix.
pub const REME_TAG: &str = " `^REME: `#";
/// The QEME suffix.
pub const QEME_TAG: &str = " `^QEME: `#";
/// The LEME suffix.
pub const LEME_TAG: &str = " `^LEME: `#";

/// Remove Growtopia colour codes: a backtick plus one character.
pub fn strip_codes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '`' {
            chars.next(); // drop the code character
        } else {
            out.push(c);
        }
    }
    out
}

/// Whether a line looks like a wheel spin.
pub fn looks_like_spin(text: &str) -> bool {
    strip_codes(text).to_lowercase().contains(SPIN_PHRASE)
}

/// A parsed spin announcement.
#[derive(Debug, Clone, Default)]
pub struct Spin {
    pub name: String,
    pub number: u32,
    pub hidden: bool,
    pub colored: bool,
    pub tags: HashMap<String, u32>,
}

/// Parse `<name> spun the wheel and got <N>`. Returns `None` when the line is
/// not a spin.
pub fn parse_spin_line(raw: &str) -> Option<Spin> {
    if !looks_like_spin(raw) {
        return None;
    }
    let clean = strip_codes(raw);
    let hidden = clean.to_lowercase().contains("hidden");
    // A colour-coded player prefix, e.g. "`6<Alice>`` spun the wheel".
    let colored = raw.contains("`0[`2REAL```0]````") || raw.contains("`0[`4FAKE```0]````")
        || raw.matches('`').count() >= 2;
    if hidden && raw.contains('`') {
        // The colour-coded [HIDDEN] prefix is machine output.
    }

    // Strip the [REAL] / [FAKE] tags before looking for the pattern.
    let mut stripped = clean.clone();
    for tag in ["[REAL]", "[FAKE]"] {
        stripped = stripped.replace(tag, "");
    }
    let stripped = stripped.trim();

    let lower = stripped.to_lowercase();
    let phrase_at = lower.find(SPIN_PHRASE)?;
    let name_part = stripped[..phrase_at].trim();
    let number_part = stripped[phrase_at + SPIN_PHRASE.len()..].trim();
    let digits: String = number_part.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || digits.len() > 3 {
        return None;
    }
    let number: u32 = digits.parse().ok()?;
    if number > 999 {
        return None;
    }

    let mut name = name_part.to_string();
    if let Some((_, tail)) = name.rsplit_once('|') {
        name = tail.to_string();
    }
    if let Some((_, tail)) = name.rsplit_once('\n') {
        name = tail.to_string();
    }
    let name = name.trim().trim_matches('<').trim_matches('>').trim().to_string();
    let mut hidden = hidden;
    if name.is_empty() || name.to_lowercase() == "hidden" {
        hidden = true;
    }
    let name = if hidden && name.is_empty() {
        "[HIDDEN]".to_string()
    } else {
        name
    };

    let mut tags = HashMap::new();
    for tag in ["REME", "QEME", "LEME"] {
        if let Some(at) = raw.to_uppercase().find(tag) {
            let tail = &raw[at + tag.len()..];
            let digits: String = tail
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() && digits.len() <= 6 {
                tags.insert(tag.to_string(), digits.parse().unwrap_or(0));
            }
        }
    }

    Some(Spin {
        name,
        number,
        hidden,
        colored,
        tags,
    })
}

fn history(ctx: &Context) -> Vec<Value> {
    ctx.config
        .get("roulette_history")
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn remember_real(ctx: &mut Context, name: &str, number: u32) {
    let mut pending = ctx
        .state
        .pending_spins
        .clone();
    pending.push(lzstate::PendingSpin {
        name: name.to_lowercase(),
        number,
        seen: lzstate::now(),
    });
    if pending.len() > 100 {
        let excess = pending.len() - 100;
        pending.drain(0..excess);
    }
    ctx.state.pending_spins = pending;
}

fn matches_real(ctx: &Context, name: &str, number: u32) -> bool {
    let window = ctx.config.get_f64("roulette_verify_window");
    let cutoff = lzstate::now() - window;
    let target = name.to_lowercase();
    ctx.state
        .pending_spins
        .iter()
        .rev()
        .find(|item| item.seen >= cutoff)
        .map(|item| item.number == number && (item.name == target || target == "[hidden]"))
        .unwrap_or(false)
}

fn record(ctx: &mut Context, spin: &Spin, source: &str, real: bool) -> Value {
    let mut entry = serde_json::Map::new();
    entry.insert("name".into(), Value::String(spin.name.clone()));
    entry.insert("number".into(), Value::from(spin.number));
    entry.insert("hidden".into(), Value::Bool(spin.hidden));
    entry.insert("real".into(), Value::Bool(real));
    entry.insert("source".into(), Value::String(source.to_string()));
    entry.insert("at".into(), Value::from(lzstate::now()));

    let mut list = history(ctx);
    list.push(Value::Object(entry.clone()));
    if list.len() > 500 {
        let excess = list.len() - 500;
        list.drain(0..excess);
    }
    ctx.config.set("roulette_history", Value::Array(list));
    Value::Object(entry)
}

fn decorated_line(ctx: &Context, entry: &Value) -> String {
    let name = entry.get("name").and_then(|v| v.as_str()).unwrap_or("?");
    let number = entry.get("number").and_then(|v| v.as_i64()).unwrap_or(0);
    let real = entry.get("real").and_then(|v| v.as_bool()).unwrap_or(false);
    let hidden = entry.get("hidden").and_then(|v| v.as_bool()).unwrap_or(false);
    let name = if hidden && ctx.config.get_bool("roulette_hidden") {
        "[HIDDEN]".to_string()
    } else {
        name.to_string()
    };
    let mut line = format!("{name} spun the wheel and got {number}");
    if real {
        line = format!("{TAG_REAL}{line}");
    } else {
        line = format!("{line}{TAG_FAKE}");
    }
    if ctx.config.get_bool("roulette_reme") {
        line.push_str(REME_TAG);
    }
    if ctx.config.get_bool("roulette_qeme") {
        line.push_str(QEME_TAG);
    }
    if ctx.config.get_bool("roulette_leme") {
        line.push_str(LEME_TAG);
    }
    line
}

/// Handle a server-side chat line. Returns the parsed spin when it matched.
pub fn on_talk(ctx: &mut Context, text: &str, source: &str) -> Option<Spin> {
    let spin = parse_spin_line(text)?;
    let real = spin.colored || matches_real(ctx, &spin.name, spin.number);
    if real {
        remember_real(ctx, &spin.name, spin.number);
    }
    let entry = record(ctx, &spin, source, real);
    let line = decorated_line(ctx, &entry);
    ctx.console(&format!("`2{line}``"));
    Some(spin)
}

/// Handle a client-side chat line.
pub fn on_client_chat(ctx: &mut Context, text: &str) -> Option<Spin> {
    on_talk(ctx, text, "client")
}

/// The `/roulette` command.
pub fn roulette(ctx: &mut Context, args: &str) {
    let parts: Vec<&str> = args.split_whitespace().collect();
    let action = parts.first().copied().unwrap_or("").to_lowercase();

    match action.as_str() {
        "fast" => {
            let value = !ctx.config.get_bool("fast_roulette");
            ctx.config.set_bool("fast_roulette", value);
            ctx.console(&format!(
                "`2FastRoulette: {}``",
                if value { "ON" } else { "OFF" }
            ));
            return;
        }
        "auto" => {
            let value = !ctx.config.get_bool("roulette_auto");
            ctx.config.set_bool("roulette_auto", value);
            ctx.console(&format!(
                "`2Auto spin: {}``",
                if value { "ON" } else { "OFF" }
            ));
            return;
        }
        "interval" => {
            if let Some(v) = parts.get(1) {
                if v.chars().all(|c| c.is_ascii_digit()) {
                    let value = v.parse::<i64>().unwrap_or(250).max(50);
                    ctx.config.set_int("roulette_interval", value);
                }
            }
            ctx.console(&format!(
                "`2Spin interval: {}ms``",
                ctx.config.get_int("roulette_interval")
            ));
            return;
        }
        "set" => {
            if parts.len() >= 3 {
                ctx.config.set_str("roulette_dialog", parts[1]);
                ctx.config.set_str("roulette_button", parts[2]);
                ctx.console(&format!("`2Spin target -> {}/{}``", parts[1], parts[2]));
            } else {
                ctx.console("`9Usage: /roulette set <dialog_name> <button>``");
            }
            return;
        }
        "real" | "fake" => {
            let value = !ctx.config.get_bool("roulette_real_fake");
            ctx.config.set_bool("roulette_real_fake", value);
            ctx.console(&format!(
                "`2Real/Fake Spin: {}``",
                if value { "ON" } else { "OFF" }
            ));
            return;
        }
        "reme" | "qeme" | "leme" => {
            let key = format!("roulette_{action}");
            let value = !ctx.config.get_bool(&key);
            ctx.config.set_bool(&key, value);
            ctx.console(&format!(
                "`2Show {} Spins: {}``",
                action.to_uppercase(),
                if value { "ON" } else { "OFF" }
            ));
            return;
        }
        "hidden" => {
            let value = !ctx.config.get_bool("roulette_hidden");
            ctx.config.set_bool("roulette_hidden", value);
            ctx.console(&format!(
                "`2Hide names in roulette logs: {}``",
                if value { "ON" } else { "OFF" }
            ));
            return;
        }
        "names" => {
            let names = ctx
                .state
                .pending_spins
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>();
            ctx.console(&format!("`2Pending names: {}``", names.join(", ")));
            return;
        }
        _ => {}
    }

    let mut d = ctx.page("lanzy_roulette", "`2Roulette``");
    d.textbox(&format!(
        "Interval: {}ms | Auto: {}",
        ctx.config.get_int("roulette_interval"),
        if ctx.config.get_bool("roulette_auto") { "ON" } else { "OFF" }
    ));
    d.textbox(&format!(
        "REAL/FAKE: {} | REME: {} | QEME: {}",
        if ctx.config.get_bool("roulette_real_fake") { "ON" } else { "OFF" },
        if ctx.config.get_bool("roulette_reme") { "ON" } else { "OFF" },
        if ctx.config.get_bool("roulette_qeme") { "ON" } else { "OFF" }
    ));
    let entries = history(ctx);
    d.textbox(&format!("Spins recorded: {}", entries.len()));
    for entry in entries.iter().rev().take(6) {
        d.textbox(&decorated_line(ctx, entry));
    }
    d.button("roulette_fast", "Toggle fast spin");
    d.button("roulette_auto", "Toggle auto spin");
    ctx.dialog("lanzy_roulette", &d);
}

/// The `/showspin` command.
pub fn showspin(ctx: &mut Context, _args: &str) {
    let entries = history(ctx);
    let mut d = ctx.page("lanzy_showspin", "`2Last Spin Per Player``");
    if entries.is_empty() {
        d.textbox("No spins recorded yet.");
    }
    let mut last: HashMap<String, Value> = HashMap::new();
    for entry in entries.iter() {
        if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
            last.insert(name.to_lowercase(), entry.clone());
        }
    }
    for entry in last.values().take(12) {
        d.textbox(&decorated_line(ctx, entry));
    }
    ctx.dialog("lanzy_showspin", &d);
}

/// Per-frame work: the auto-spin ticker.
pub fn tick(ctx: &mut Context) {
    if !ctx.config.get_bool("roulette_auto") {
        return;
    }
    let interval = ctx.config.get_f64("roulette_interval").max(50.0);
    let now = lzstate::now();
    let last = ctx.state.timers.get("roulette").copied().unwrap_or(0.0);
    if now - last < interval / 1000.0 {
        return;
    }
    ctx.state.timers.insert("roulette".into(), now);

    let dialog = ctx.config.get_str("roulette_dialog");
    let button = ctx.config.get_str("roulette_button");
    if dialog.is_empty() || button.is_empty() {
        return;
    }
    ctx.dialog_return(&[("dialog_name", dialog), ("button", button)]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plain_spin() {
        let spin = parse_spin_line("Alice spun the wheel and got 42").expect("parse");
        assert_eq!(spin.name, "Alice");
        assert_eq!(spin.number, 42);
        assert!(!spin.hidden);
    }

    #[test]
    fn parses_a_coloured_spin() {
        let spin = parse_spin_line("`6<Bob>`` spun the wheel and got 7").expect("parse");
        assert_eq!(spin.number, 7);
        assert!(spin.colored);
    }

    #[test]
    fn rejects_a_non_spin() {
        assert!(parse_spin_line("hello world").is_none());
    }

    #[test]
    fn strips_colour_codes() {
        assert_eq!(strip_codes("`2hello`"), "hello");
    }
}
