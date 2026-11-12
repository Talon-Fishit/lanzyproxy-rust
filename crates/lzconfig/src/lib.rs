//! Persistent settings.
//!
//! A JSON document next to the binary, with three save slots and an autosave
//! that writes after every change. Values the build does not know are kept, so
//! a newer build's settings survive a round trip through an older one.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// How many save slots the config offers.
pub const SLOTS: usize = 3;

/// The configuration document.
#[derive(Debug, Clone, Default)]
pub struct Config {
    values: Map<String, Value>,
    path: PathBuf,
    slot: usize,
    autosave: bool,
}

impl Config {
    /// Load a config file, filling in defaults for anything missing.
    pub fn load(path: &Path) -> Config {
        let mut cfg = Config {
            values: Map::new(),
            path: path.to_path_buf(),
            slot: 0,
            autosave: true,
        };
        if let Ok(text) = fs::read_to_string(path) {
            if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&text) {
                cfg.values = map;
            }
        }
        cfg.apply_defaults();
        cfg
    }

    /// A config that is never written to disk.
    pub fn ephemeral() -> Config {
        let mut cfg = Config::default();
        cfg.apply_defaults();
        cfg
    }

    /// The default settings, applied to any key the file does not define.
    pub fn defaults() -> Map<String, Value> {
        let mut m = Map::new();
        let mut set = |k: &str, v: Value| {
            m.insert(k.to_string(), v);
        };

        // core
        set("proxy_host", Value::String("127.0.0.1".into()));
        set("proxy_port", Value::from(17198));
        // The reference build uses 17725. The older 17091 is not what the
        // game listens on any more.
        set("upstream_host", Value::String("www.growtopia1.com".into()));
        set("upstream_port", Value::from(17725));
        // The two front doors the game publishes. The proxy measures each one
        // and keeps the faster.
        set("upstream_alt_host", Value::String("www.growtopia2.com".into()));
        set("upstream_alt_port", Value::from(17725));
        set("upstream_probe_ms", Value::from(1500));
        // An optional SOCKS5 proxy for the game's UDP traffic.
        set("socks5_host", Value::String(String::new()));
        set("socks5_port", Value::from(0));
        set("socks5_user", Value::String(String::new()));
        set("socks5_pass", Value::String(String::new()));
        // The account the proxy logs in with.
        set("login_username", Value::String(String::new()));
        set("login_password", Value::String(String::new()));
        set("spoof_http_port", Value::from(80));
        set("auto_save", Value::Bool(true));
        set("hidden_mode", Value::Bool(false));

        // movement
        set("speed", Value::from(1.0));
        set("jump", Value::from(1.0));
        set("gravity", Value::from(1.0));
        set("fall_speed", Value::from(1.0));
        set("water_speed", Value::from(1.0));
        set("unlimited_jump", Value::Bool(false));
        set("ghost_mode", Value::Bool(false));
        set("antigravity", Value::Bool(false));
        set("antipunch", Value::Bool(false));
        set("antilava", Value::Bool(false));
        set("frozen", Value::Bool(false));
        set("floating", Value::Bool(false));

        // visuals
        set("visual_clothes", Value::Array(vec![]));
        set("visual_title", Value::String(String::new()));
        set("visual_name", Value::String(String::new()));
        set("skin_color", Value::from(-1));
        set("blink_skin", Value::Bool(false));
        set("country", Value::String(String::new()));
        set("weather", Value::String(String::new()));
        set("ui_color", Value::String(String::new()));
        set("hide_nicknames", Value::Bool(false));
        set("hide_clothes", Value::Bool(false));
        set("glass_background", Value::Bool(false));

        // automation
        for key in [
            "autofarm",
            "autoplant",
            "autoharvest",
            "autocollect",
            "autocook",
            "autofish",
            "providers",
            "autocrime",
            "auto_accept",
            "autolock",
        ] {
            set(key, Value::Bool(false));
        }
        set("automsg", Value::String(String::new()));
        set("automsg_delay", Value::from(5.0));
        let mut intervals = Map::new();
        for (k, v) in [
            ("autofarm", 120),
            ("autoplant", 150),
            ("autoharvest", 200),
            ("autocollect", 100),
            ("autocook", 500),
            ("autofish", 900),
            ("providers", 250),
        ] {
            intervals.insert(k.to_string(), Value::from(v));
        }
        set("intervals", Value::Object(intervals));

        // moderation
        set("moddetect", Value::Bool(true));
        set("wrench_mode", Value::String("pull".into()));
        set("autoban_words", Value::Array(vec![]));
        set("autoban_players", Value::Array(vec![]));
        set("autopull_players", Value::Array(vec![]));
        set("auto_on_join", Value::Bool(false));
        set("wipe_on_mod", Value::Bool(false));

        // navigation
        set("pathfind_delay", Value::from(80));
        set("pathfind_speed", Value::from(1.0));
        set("saved_worlds", Value::Object(Map::new()));
        set("autohost_pos", Value::Object(Map::new()));

        // economy
        set("fast_vend", Value::Bool(false));
        set("fast_vend_amount", Value::from(1));
        set("shop_bulk", Value::from(1));
        set("saved_items", Value::Array(vec![]));

        // casino
        set("fast_roulette", Value::Bool(false));
        set("roulette_auto", Value::Bool(false));
        set("roulette_interval", Value::from(250));
        set("roulette_dialog", Value::String("roulette".into()));
        set("roulette_button", Value::String("spin".into()));
        set("roulette_history", Value::Array(vec![]));
        set("roulette_last", Value::Object(Map::new()));
        set("roulette_export", Value::Bool(false));
        set("roulette_real_fake", Value::Bool(true));
        set("roulette_reme", Value::Bool(true));
        set("roulette_qeme", Value::Bool(true));

        m
    }

    fn apply_defaults(&mut self) {
        for (k, v) in Self::defaults() {
            self.values.entry(k).or_insert(v);
        }
        let autosave = self
            .values
            .get("auto_save")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        self.autosave = autosave;
    }

    /// Read a value by key.
    pub fn get(&self, key: &str) -> Value {
        self.values.get(key).cloned().unwrap_or(Value::Null)
    }

    /// Read a string value.
    pub fn get_str(&self, key: &str) -> String {
        self.get(key).as_str().unwrap_or_default().to_string()
    }

    /// Read an integer value.
    pub fn get_int(&self, key: &str) -> i64 {
        self.get(key).as_i64().unwrap_or(0)
    }

    /// Read a float value.
    pub fn get_f64(&self, key: &str) -> f64 {
        self.get(key).as_f64().unwrap_or(0.0)
    }

    /// Read a boolean value.
    pub fn get_bool(&self, key: &str) -> bool {
        self.get(key).as_bool().unwrap_or(false)
    }

    /// Write a value, saving immediately when autosave is on.
    pub fn set(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value);
        if self.autosave {
            self.save();
        }
    }

    /// Write a boolean.
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.set(key, Value::Bool(value));
    }

    /// Write an integer.
    pub fn set_int(&mut self, key: &str, value: i64) {
        self.set(key, Value::from(value));
    }

    /// Write a string.
    pub fn set_str(&mut self, key: &str, value: &str) {
        self.set(key, Value::String(value.to_string()));
    }

    /// The current save slot.
    pub fn slot(&self) -> usize {
        self.slot
    }

    /// Switch to another save slot.
    pub fn set_slot(&mut self, slot: usize) {
        self.slot = slot % SLOTS;
        self.save();
    }

    /// Every stored key.
    pub fn keys(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    /// The raw JSON document.
    pub fn raw(&self) -> &Map<String, Value> {
        &self.values
    }

    /// Write the config to disk.
    pub fn save(&self) -> std::io::Result<()> {
        if self.path.as_os_str().is_empty() {
            return Ok(());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(&Value::Object(self.values.clone()))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&self.path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_present() {
        let cfg = Config::ephemeral();
        assert_eq!(cfg.get_int("proxy_port"), 17198);
        assert_eq!(cfg.get_str("upstream_host"), "www.growtopia1.com");
        assert_eq!(cfg.get_int("upstream_port"), 17725);
        assert!(cfg.get_bool("auto_save"));
    }

    #[test]
    fn unknown_keys_survive_a_round_trip() {
        let dir = std::env::temp_dir().join("lzconfig-test");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        let mut cfg = Config::load(&path);
        cfg.set("future_key", Value::from(7));
        cfg.save().unwrap();
        let again = Config::load(&path);
        assert_eq!(again.get_int("future_key"), 7);
        let _ = fs::remove_file(&path);
    }
}
