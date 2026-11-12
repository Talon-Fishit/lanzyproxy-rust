//! In-game dialog markup.
//!
//! Growtopia renders dialogs from newline-separated markup that the server
//! sends through `OnDialogRequest`. The same markup backs every page in
//! LanzyProxy.
//!
//! The directive names and their argument layouts were recovered from the
//! reference build's own strings, so a page rendered here uses the same
//! spelling the game expects.
//!
//! Directives:
//! ```text
//! add_label|text|align|width|
//! add_label_with_icon|text|align|width|icon|
//! add_textbox|text|align|
//! add_textbox_custom|text|align|
//! add_smalltext|text|
//! add_word|text|
//! add_spacer|small|medium|large|
//! add_button|name|label|noflags|item_id|item_count|
//! add_button_with_icon|name|label|noflags|item_id|item_count|icon|
//! add_text_input|name|text|default|
//! add_text_input_password|name|text|default|
//! add_checkbox|name|label|checked|
//! add_checkicon|name|label|checked|icon|
//! add_item_picker|name|label|
//! add_player_info|netid|
//! add_player_info_small|netid|
//! add_custom_label|text|align|width|
//! add_custom_textbox|text|align|
//! add_custom_button|name|label|
//! add_custom_margin|size|
//! add_custom_break|
//! add_key_share|
//! add_quick_exit|
//! add_puzzle_captcha|
//! add_url_button|name|label|url|
//! add_friend_image_label_button|name|label|friend_id|
//! add_label_with_icon_button|name|label|icon|
//! add_label_with_icon_button_list|names|labels|icons|
//! start_custom_tabs|
//! end_custom_tabs|
//! end_dialog|name|button1|button2|
//! ```

/// A dialog under construction.
#[derive(Debug, Clone, Default)]
pub struct Dialog {
    name: String,
    lines: Vec<String>,
}

impl Dialog {
    /// Start a dialog. `title` becomes the first label when given.
    pub fn new(name: &str, title: Option<&str>) -> Self {
        let mut d = Dialog {
            name: name.to_string(),
            lines: Vec::new(),
        };
        if let Some(t) = title {
            d.label(t);
        }
        d
    }

    /// A dialog with a title.
    pub fn titled(name: &str, title: &str) -> Self {
        Dialog::new(name, Some(title))
    }

    /// The dialog's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// A text label.
    pub fn label(&mut self, text: &str) -> &mut Self {
        self.label_at(text, "left", 0)
    }

    /// A text label with an alignment and a width.
    pub fn label_at(&mut self, text: &str, align: &str, width: i32) -> &mut Self {
        self.lines
            .push(format!("add_label|{text}|{align}|{width}|"));
        self
    }

    /// A text box.
    pub fn textbox(&mut self, text: &str) -> &mut Self {
        self.textbox_at(text, "left")
    }

    /// A text box with an alignment.
    pub fn textbox_at(&mut self, text: &str, align: &str) -> &mut Self {
        self.lines.push(format!("add_textbox|{text}|{align}|"));
        self
    }

    /// Small print.
    pub fn small(&mut self, text: &str) -> &mut Self {
        self.lines.push(format!("add_smalltext|{text}|"));
        self
    }

    /// Vertical space.
    pub fn spacer(&mut self, size: &str) -> &mut Self {
        self.lines.push(format!("add_spacer|{size}|"));
        self
    }

    /// A button.
    pub fn button(&mut self, name: &str, label: &str) -> &mut Self {
        self.button_item(name, label, 0, 0)
    }

    /// A button carrying an item id and a count.
    pub fn button_item(&mut self, name: &str, label: &str, item: i32, count: i32) -> &mut Self {
        self.lines.push(format!(
            "add_button|{name}|{label}|noflags|{item}|{count}|"
        ));
        self
    }

    /// A button that reads as on or off.
    pub fn toggle(&mut self, name: &str, label: &str, enabled: bool) -> &mut Self {
        let state = if enabled { "`2ON``" } else { "`4OFF``" };
        self.button(name, &format!("{label}: {state}"))
    }

    /// A raw markup line, for directives this builder does not wrap.
    pub fn raw(&mut self, line: &str) -> &mut Self {
        self.lines.push(line.to_string());
        self
    }

    // -- the remaining directives the reference build emits ---------------

    /// A label with an icon.
    pub fn label_with_icon(&mut self, text: &str, icon: &str) -> &mut Self {
        self.raw(&format!("add_label_with_icon|{text}|left|0|{icon}|"));
        self
    }

    /// A text box with a custom style.
    pub fn textbox_custom(&mut self, text: &str) -> &mut Self {
        self.raw(&format!("add_custom_textbox|{text}|left|"));
        self
    }

    /// A single word, for the word-based layouts.
    pub fn word(&mut self, text: &str) -> &mut Self {
        self.raw(&format!("add_word|{text}|"));
        self
    }

    /// A button with an icon.
    pub fn button_with_icon(&mut self, name: &str, label: &str, icon: &str) -> &mut Self {
        self.raw(&format!(
            "add_button_with_icon|{name}|{label}|noflags|0|0|{icon}|"
        ));
        self
    }

    /// A text input field.
    pub fn text_input(&mut self, name: &str, text: &str, default: &str) -> &mut Self {
        self.raw(&format!("add_text_input|{name}|{text}|{default}|"));
        self
    }

    /// A password input field.
    pub fn text_input_password(&mut self, name: &str, text: &str, default: &str) -> &mut Self {
        self.raw(&format!("add_text_input_password|{name}|{text}|{default}|"));
        self
    }

    /// A checkbox.
    pub fn checkbox(&mut self, name: &str, label: &str, checked: bool) -> &mut Self {
        self.raw(&format!(
            "add_checkbox|{name}|{label}|{}|",
            if checked { "1" } else { "0" }
        ));
        self
    }

    /// A checkbox with an icon.
    pub fn checkicon(&mut self, name: &str, label: &str, checked: bool, icon: &str) -> &mut Self {
        self.raw(&format!(
            "add_checkicon|{name}|{label}|{}|{icon}|",
            if checked { "1" } else { "0" }
        ));
        self
    }

    /// The item picker.
    pub fn item_picker(&mut self, name: &str, label: &str) -> &mut Self {
        self.raw(&format!("add_item_picker|{name}|{label}|"));
        self
    }

    /// A player info card.
    pub fn player_info(&mut self, netid: i32) -> &mut Self {
        self.raw(&format!("add_player_info|{netid}|"));
        self
    }

    /// A small player info card.
    pub fn player_info_small(&mut self, netid: i32) -> &mut Self {
        self.raw(&format!("add_player_info_small|{netid}|"));
        self
    }

    /// A label with a custom style.
    pub fn custom_label(&mut self, text: &str) -> &mut Self {
        self.raw(&format!("add_custom_label|{text}|left|0|"));
        self
    }

    /// A button with a custom style.
    pub fn custom_button(&mut self, name: &str, label: &str) -> &mut Self {
        self.raw(&format!("add_custom_button|{name}|{label}|"));
        self
    }

    /// A custom margin.
    pub fn custom_margin(&mut self, size: &str) -> &mut Self {
        self.raw(&format!("add_custom_margin|{size}|"));
        self
    }

    /// A line break in a custom layout.
    pub fn custom_break(&mut self) -> &mut Self {
        self.raw("add_custom_break|");
        self
    }

    /// The key share row.
    pub fn key_share(&mut self) -> &mut Self {
        self.raw("add_key_share|");
        self
    }

    /// The quick exit row.
    pub fn quick_exit(&mut self) -> &mut Self {
        self.raw("add_quick_exit|");
        self
    }

    /// The puzzle captcha row.
    pub fn puzzle_captcha(&mut self) -> &mut Self {
        self.raw("add_puzzle_captcha|");
        self
    }

    /// A button that opens a URL.
    pub fn url_button(&mut self, name: &str, label: &str, url: &str) -> &mut Self {
        self.raw(&format!("add_url_button|{name}|{label}|{url}|"));
        self
    }

    /// A friend image button.
    pub fn friend_image_label_button(&mut self, name: &str, label: &str, friend_id: i32) -> &mut Self {
        self.raw(&format!(
            "add_friend_image_label_button|{name}|{label}|{friend_id}|"
        ));
        self
    }

    /// A label that is also a button.
    pub fn label_with_icon_button(&mut self, name: &str, label: &str, icon: &str) -> &mut Self {
        self.raw(&format!(
            "add_label_with_icon_button|{name}|{label}|{icon}|"
        ));
        self
    }

    /// A list of labels that are also buttons.
    pub fn label_with_icon_button_list(
        &mut self,
        names: &[&str],
        labels: &[&str],
        icons: &[&str],
    ) -> &mut Self {
        self.raw(&format!(
            "add_label_with_icon_button_list|{}|{}|{}|",
            names.join(","),
            labels.join(","),
            icons.join(",")
        ));
        self
    }

    /// Open a custom tab section.
    pub fn start_custom_tabs(&mut self) -> &mut Self {
        self.raw("start_custom_tabs|");
        self
    }

    /// Close a custom tab section.
    pub fn end_custom_tabs(&mut self) -> &mut Self {
        self.raw("end_custom_tabs|");
        self
    }

    /// The finished markup, ending with `end_dialog`.
    pub fn build(&self, close: &str, alternative: &str) -> String {
        let mut out = self.lines.join("\n");
        out.push('\n');
        out.push_str(&format!("end_dialog|{}|{close}|{alternative}|", self.name));
        out
    }
}

/// A dialog with the usual close and cancel buttons.
pub fn simple(name: &str, title: &str) -> String {
    Dialog::titled(name, title).build("OK", "Cancel")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_ends_with_end_dialog() {
        let mut d = Dialog::titled("shop", "Shop");
        d.small("hello").spacer("small").button_item("buy", "Buy", 1, 2);
        let markup = d.build("OK", "Cancel");
        assert!(markup.starts_with("add_label|Shop|left|0|"));
        assert!(markup.contains("add_smalltext|hello|"));
        assert!(markup.contains("add_button|buy|Buy|noflags|1|2|"));
        assert!(markup.ends_with("end_dialog|shop|OK|Cancel|"));
    }

    #[test]
    fn every_reference_directive_is_available() {
        // The reference build's own directive list.
        let mut d = Dialog::new("x", None);
        d.label_with_icon("a", "i")
            .textbox_custom("b")
            .word("c")
            .button_with_icon("n", "l", "i")
            .text_input("n", "t", "")
            .text_input_password("n", "t", "")
            .checkbox("n", "l", true)
            .checkicon("n", "l", false, "i")
            .item_picker("n", "l")
            .player_info(1)
            .player_info_small(2)
            .custom_label("t")
            .custom_button("n", "l")
            .custom_margin("small")
            .custom_break()
            .key_share()
            .quick_exit()
            .puzzle_captcha()
            .url_button("n", "l", "https://example.com")
            .friend_image_label_button("n", "l", 7)
            .label_with_icon_button("n", "l", "i")
            .label_with_icon_button_list(&["a"], &["b"], &["c"])
            .start_custom_tabs()
            .end_custom_tabs();
        let markup = d.build("OK", "Cancel");
        for directive in [
            "add_label_with_icon|", "add_custom_textbox|", "add_word|",
            "add_button_with_icon|", "add_text_input|", "add_text_input_password|",
            "add_checkbox|", "add_checkicon|", "add_item_picker|", "add_player_info|",
            "add_player_info_small|", "add_custom_label|", "add_custom_button|",
            "add_custom_margin|", "add_custom_break|", "add_key_share|",
            "add_quick_exit|", "add_puzzle_captcha|", "add_url_button|",
            "add_friend_image_label_button|", "add_label_with_icon_button|",
            "add_label_with_icon_button_list|", "start_custom_tabs|", "end_custom_tabs|",
        ] {
            assert!(markup.contains(directive), "missing {directive}");
        }
    }

    #[test]
    fn toggle_shows_state() {
        let mut d = Dialog::new("cfg", None);
        d.toggle("moddetect", "ModDetect", true);
        assert!(d.build("OK", "Cancel").contains("ModDetect: `2ON``"));
    }
}
