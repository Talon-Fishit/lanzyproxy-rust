//! Utility commands: the command list, settings, slots, logs, status.

use serde_json::Value;

use crate::common::toggle;
use crate::{Context, REGISTRY};

pub fn proxy(ctx: &mut Context, args: &str) {
    let mut names: Vec<&str> = REGISTRY.keys().copied().collect();
    names.sort_unstable();
    let page = args.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
    let per_page = 9usize;
    let total = names.len().div_ceil(per_page);
    let mut d = ctx.page("lanzy_commands", &format!("`2Proxy Commands ({}/{})``", page + 1, total));
    for name in names.iter().skip(page * per_page).take(per_page) {
        let summary = REGISTRY[*name].summary;
        d.textbox(&format!("/{name} - {summary}"));
    }
    d.textbox(&format!("Total commands: {}", names.len()));
    if page + 1 < total {
        d.button(&format!("cmd_page_{}", page + 2), "Next page");
    }
    ctx.dialog("lanzy_commands", &d);
}

pub fn settings(ctx: &mut Context, args: &str) {
    let args = args.trim();
    if let Some(rest) = args.strip_prefix("save") {
        let slot = rest.trim().to_string();
        let slot = if slot.is_empty() { "1".to_string() } else { slot };
        ctx.config.set_slot(slot.parse::<usize>().unwrap_or(0));
        ctx.console(&format!("`2Settings saved to slot {slot}``"));
        return;
    }
    if let Some(rest) = args.strip_prefix("load") {
        let slot = rest.trim().to_string();
        let slot = if slot.is_empty() { "1".to_string() } else { slot };
        ctx.config.set_slot(slot.parse::<usize>().unwrap_or(0));
        ctx.console(&format!("`2Settings loaded from slot {slot}``"));
        return;
    }
    let mut d = ctx.page("lanzy_settings", "`2Proxy Settings``");
    d.textbox(&format!(
        "Proxy: {}:{}",
        ctx.config.get_str("proxy_host"),
        ctx.config.get_int("proxy_port")
    ));
    d.textbox(&format!(
        "Upstream: {}:{}",
        ctx.config.get_str("upstream_host"),
        ctx.config.get_int("upstream_port")
    ));
    d.textbox(&format!(
        "Autosave: {}",
        if ctx.config.get_bool("auto_save") { "ON" } else { "OFF" }
    ));
    d.button("cfg_save", "Save to slot 1");
    d.button("cfg_load", "Load slot 1");
    d.button("cfg_reset", "Reset settings");
    ctx.dialog("lanzy_settings", &d);
}

pub fn options(ctx: &mut Context, _args: &str) {
    crate::options::render(ctx);
}

pub fn keep(ctx: &mut Context, _args: &str) {
    ctx.config.set_slot(0);
    ctx.console("`2Settings saved to slot 'main'``");
}

pub fn reset(ctx: &mut Context, _args: &str) {
    ctx.config
        .set("autohost_pos", Value::Object(serde_json::Map::new()));
    ctx.console("`2Autohost positions cleared.``");
}

pub fn hotkey(ctx: &mut Context, args: &str) {
    let mut hotkeys = ctx.config.get("hotkeys");
    if !hotkeys.is_object() {
        hotkeys = Value::Object(serde_json::Map::new());
    }
    let map = hotkeys.as_object_mut().unwrap();
    if !args.trim().is_empty() {
        let (key, cmd) = args.trim().split_once(' ').unwrap_or((args.trim(), ""));
        if !key.is_empty() && !cmd.is_empty() {
            map.insert(key.to_uppercase(), Value::String(cmd.to_string()));
            ctx.config.set("hotkeys", hotkeys);
            ctx.console(&format!("`2Hotkey {} -> {}``", key.to_uppercase(), cmd));
            return;
        }
    }
    let mut d = ctx.page("lanzy_hotkey", "`2Custom Hotkeys``");
    if map.is_empty() {
        d.textbox("No hotkeys configured.");
        d.textbox("Format: /hotkey F1 /farm");
    }
    for (key, cmd) in map.iter().take(8) {
        d.textbox(&format!("{key} -> {}", cmd.as_str().unwrap_or_default()));
    }
    ctx.dialog("lanzy_hotkey", &d);
}

pub fn logs(ctx: &mut Context, _args: &str) {
    let recent: Vec<String> = ctx
        .state
        .logs
        .iter()
        .rev()
        .take(12)
        .map(|(_, m)| m.chars().take(90).collect())
        .collect();
    if recent.is_empty() {
        ctx.console("`9No log entries.``");
        return;
    }
    let mut d = ctx.page("lanzy_logs", "`2Proxy Logs``");
    for line in recent.iter().rev() {
        d.textbox(line);
    }
    ctx.dialog("lanzy_logs", &d);
}

pub fn server(ctx: &mut Context, _args: &str) {
    let mut d = ctx.page("lanzy_server", "`2Server Info``");
    d.textbox(&format!(
        "Local proxy: {}:{}",
        ctx.config.get_str("proxy_host"),
        ctx.config.get_int("proxy_port")
    ));
    d.textbox(&format!(
        "Upstream: {}:{}",
        ctx.config.get_str("upstream_host"),
        ctx.config.get_int("upstream_port")
    ));
    d.textbox(&format!(
        "HTTP spoofer port: {}",
        ctx.config.get_int("spoof_http_port")
    ));
    d.textbox(&format!(
        "Connection: {}",
        if ctx.state.connected { "`2connected``" } else { "`4offline``" }
    ));
    ctx.dialog("lanzy_server", &d);
}

pub fn logoff(ctx: &mut Context, _args: &str) {
    ctx.input("action|quit");
    ctx.console("`2Log off requested.``");
}

pub fn calc(ctx: &mut Context, args: &str) {
    let expr = args.trim();
    if expr.is_empty() {
        ctx.console("`9Usage: /calc 10+6``");
        return;
    }
    match eval(expr) {
        Ok(value) => ctx.console(&format!("`2{expr} = {value}``")),
        Err(e) => ctx.console(&format!("`4Calc error: {e}``")),
    }
}

pub fn switch(ctx: &mut Context, args: &str) {
    let mut accounts = ctx.config.get("accounts");
    if !accounts.is_array() {
        accounts = Value::Array(vec![]);
    }
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() >= 2 {
        let mut entry = serde_json::Map::new();
        entry.insert("growid".into(), Value::String(parts[0].into()));
        entry.insert("password".into(), Value::String(parts[1].into()));
        accounts.as_array_mut().unwrap().push(Value::Object(entry));
        ctx.config.set("accounts", accounts);
        ctx.console(&format!("`2Account {} saved.``", parts[0]));
        return;
    }
    let count = accounts.as_array().map(|a| a.len()).unwrap_or(0);
    let mut d = ctx.page("lanzy_switch", "`2Account Switch``");
    d.textbox(&format!("Saved accounts: {count}"));
    if let Some(list) = accounts.as_array() {
        for (index, account) in list.iter().take(8).enumerate() {
            let name = account
                .get("growid")
                .and_then(|v| v.as_str())
                .unwrap_or("account")
                .to_string();
            d.button(&format!("acc_{index}"), &name);
        }
    }
    d.textbox("Add: /switch <growid> <password>");
    ctx.dialog("lanzy_switch", &d);
}

pub fn status(ctx: &mut Context, _args: &str) {
    let mut d = ctx.page("lanzy_status", "`2Proxy Status``");
    d.textbox(&format!(
        "Connection: {}",
        if ctx.state.connected { "`2connected``" } else { "`4disconnected``" }
    ));
    d.textbox(&format!("World: {}", ctx.state.world_name));
    d.textbox(&format!(
        "Position: {},{}",
        ctx.state.tile_x, ctx.state.tile_y
    ));
    d.textbox(&format!(
        "Players tracked: {} | tiles: {}",
        ctx.state.players.len(),
        ctx.state.tiles.len()
    ));
    d.textbox(&format!("Commands loaded: {}", REGISTRY.len()));
    ctx.dialog("lanzy_status", &d);
}

/// Toggle helper kept for symmetry with the Python feature modules.
pub fn toggle_setting(ctx: &mut Context, key: &str, label: &str) {
    toggle(ctx, key, label);
}

// ---------------------------------------------------------------------------
// A small arithmetic expression evaluator
// ---------------------------------------------------------------------------

/// Evaluate `+ - * / % ^` over numbers.
pub fn eval(expr: &str) -> Result<f64, String> {
    let mut parser = ExprParser {
        bytes: expr.as_bytes(),
        pos: 0,
    };
    let value = parser.expression()?;
    parser.skip_ws();
    if parser.pos != parser.bytes.len() {
        return Err("unexpected trailing input".into());
    }
    Ok(value)
}

struct ExprParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ExprParser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn expression(&mut self) -> Result<f64, String> {
        let mut value = self.term()?;
        loop {
            self.skip_ws();
            match self.bytes.get(self.pos) {
                Some(b'+') => {
                    self.pos += 1;
                    value += self.term()?;
                }
                Some(b'-') => {
                    self.pos += 1;
                    value -= self.term()?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut value = self.power()?;
        loop {
            self.skip_ws();
            match self.bytes.get(self.pos) {
                Some(b'*') => {
                    self.pos += 1;
                    value *= self.power()?;
                }
                Some(b'/') => {
                    self.pos += 1;
                    let divisor = self.power()?;
                    if divisor == 0.0 {
                        return Err("division by zero".into());
                    }
                    value /= divisor;
                }
                Some(b'%') => {
                    self.pos += 1;
                    let divisor = self.power()?;
                    if divisor == 0.0 {
                        return Err("modulo by zero".into());
                    }
                    value %= divisor;
                }
                _ => return Ok(value),
            }
        }
    }

    fn power(&mut self) -> Result<f64, String> {
        let base = self.unary()?;
        self.skip_ws();
        if self.bytes.get(self.pos) == Some(&b'^') {
            self.pos += 1;
            let exponent = self.power()?;
            return Ok(base.powf(exponent));
        }
        Ok(base)
    }

    fn unary(&mut self) -> Result<f64, String> {
        self.skip_ws();
        match self.bytes.get(self.pos) {
            Some(b'-') => {
                self.pos += 1;
                Ok(-self.unary()?)
            }
            Some(b'+') => {
                self.pos += 1;
                self.unary()
            }
            _ => self.number(),
        }
    }

    fn number(&mut self) -> Result<f64, String> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'.')
        {
            self.pos += 1;
        }
        if start == self.pos {
            return Err("expected a number".into());
        }
        std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| "bad number".to_string())?
            .parse::<f64>()
            .map_err(|_| "bad number".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_arithmetic() {
        assert_eq!(eval("10+6").unwrap(), 16.0);
        assert_eq!(eval("2*3+4").unwrap(), 10.0);
        assert_eq!(eval("2^10").unwrap(), 1024.0);
        assert_eq!(eval("-5+2").unwrap(), -3.0);
        assert_eq!(eval("7%4").unwrap(), 3.0);
    }

    #[test]
    fn rejects_garbage() {
        assert!(eval("abc").is_err());
        assert!(eval("1/0").is_err());
    }
}
