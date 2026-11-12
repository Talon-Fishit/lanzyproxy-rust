//! The options menu.
//!
//! The reference build registers seventeen options under `/options`. They are
//! reproduced here with the same labels and the same aliases, so a page
//! opened from this menu reads the way the original's does.

use crate::common::toggle;
use crate::Context;

/// One entry of the options menu.
pub struct MenuOption {
    /// The label as the original shows it.
    pub label: &'static str,
    /// The setting the entry drives, if it drives one.
    pub key: Option<&'static str>,
    /// How the entry behaves.
    pub kind: Kind,
}

/// What an option entry does.
pub enum Kind {
    /// A toggle shown with its current state.
    Toggle,
    /// A button that opens another page.
    Page(&'static str),
    /// A button that performs an action.
    Action(&'static str),
    /// A row the original keeps hidden behind the `a` colour code.
    Hidden(&'static str),
}

/// Every option, in the order the original lists them.
pub static OPTIONS: &[MenuOption] = &[
    MenuOption { label: "Fast Mag/Gaia/UT Retrieve", key: None, kind: Kind::Page("lanzy_options") },
    MenuOption {
        label: "\"You need to be Level to break that\" Text",
        key: None,
        kind: Kind::Hidden("`aHide`` \"You need to be Level to break that\" Text"),
    },
    MenuOption { label: "Antipunch", key: Some("antipunch"), kind: Kind::Toggle },
    MenuOption { label: "Anti Lava Damage", key: Some("antilava"), kind: Kind::Toggle },
    MenuOption { label: "Pathfinding Toggle", key: Some("pathfind_toggle"), kind: Kind::Toggle },
    MenuOption { label: "Autofish", key: Some("autofish"), kind: Kind::Toggle },
    MenuOption { label: "Consumables Autobanner", key: Some("auto_banner"), kind: Kind::Toggle },
    MenuOption { label: "REME Number Spin", key: Some("roulette_reme"), kind: Kind::Toggle },
    MenuOption { label: "Real/Fake Spin", key: Some("roulette_real_fake"), kind: Kind::Toggle },
    MenuOption { label: "Last Roulette Spin", key: None, kind: Kind::Action("showspin") },
    MenuOption { label: "Ping Latency", key: None, kind: Kind::Action("ping") },
    MenuOption { label: "Collected Gem Count", key: None, kind: Kind::Action("gems") },
    MenuOption { label: "Public Entrances/Gates", key: None, kind: Kind::Action("gates") },
    MenuOption { label: "Private Messages", key: None, kind: Kind::Hidden("`aHide`` Private Messages") },
    MenuOption { label: "Auto Mod Detect", key: Some("moddetect"), kind: Kind::Toggle },
    MenuOption { label: "Mod Detect Settings", key: None, kind: Kind::Page("lanzy_moddetect") },
    MenuOption { label: "Main Options Page", key: None, kind: Kind::Page("lanzy_settings") },
];

/// Render the options menu.
pub fn render(ctx: &mut Context) {
    let mut d = ctx.page("lanzy_options", "`2Options``");
    for (index, option) in OPTIONS.iter().enumerate() {
        let name = format!("opt_{index}");
        match option.kind {
            Kind::Toggle => {
                let key = option.key.unwrap_or_default();
                let enabled = ctx.config.get_bool(key);
                let state = if enabled { "`2ON``" } else { "`4OFF``" };
                d.button(&name, &format!("{}: {state}", option.label));
            }
            Kind::Page(page) => {
                d.button(&name, option.label);
                let _ = page;
            }
            Kind::Action(action) => {
                d.button(&name, option.label);
                let _ = action;
            }
            Kind::Hidden(text) => {
                d.raw(&format!("add_label|{text}|left|0|"));
            }
        }
    }
    d.spacer("small");
    d.small("Options are stored in the config and survive a restart.");
    ctx.dialog("lanzy_options", &d);
}

/// Apply an option by its index, as the buttons on the page do.
pub fn apply(ctx: &mut Context, index: usize) {
    let Some(option) = OPTIONS.get(index) else {
        return;
    };
    match option.kind {
        Kind::Toggle => {
            if let Some(key) = option.key {
                toggle(ctx, key, option.label);
            }
        }
        Kind::Page(page) => match page {
            "lanzy_settings" => crate::utility::settings(ctx, ""),
            "lanzy_options" => render(ctx),
            "lanzy_moddetect" => crate::moderation::moddetect(ctx, ""),
            _ => render(ctx),
        },
        Kind::Action(action) => match action {
            "showspin" => crate::casino::showspin(ctx, ""),
            "ping" => {
                let latency = ctx.state.last_ping_ms;
                ctx.console(&format!("`2Ping latency: {latency}ms``"));
            }
            "gems" => {
                ctx.console(&format!("`2Collected gems: {}``", ctx.state.gems));
            }
            "gates" => {
                let gates = ctx.state.gates_open;
                ctx.console(&format!("`2Public entrances/gates open: {gates}``"));
            }
            _ => {}
        },
        Kind::Hidden(_) => {}
    }
}

/// How many options the menu holds.
pub fn count() -> usize {
    OPTIONS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_matches_the_reference_count() {
        assert_eq!(count(), 17);
    }
}
