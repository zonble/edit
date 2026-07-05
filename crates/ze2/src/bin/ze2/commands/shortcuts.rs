// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::input::{InputKey, InputKeyMod, kbmod, vk};

use super::{Command, CommandArgs, CommandBarShortcut, CommandFocusTarget, CommandInvocation};

struct InsertShortcut {
    modifiers: InputKeyMod,
    key: InputKey,
    text: &'static str,
}

// The only key-to-command mapping still hardcoded is the CJK insert shortcuts
// (Alt+punctuation/number keys). Every other default binding lives in the
// profile, loaded into State::key_bindings, so the profile is the single source
// of truth for them.
pub fn command_invocation_from_shortcut(key: InputKey) -> Option<CommandInvocation> {
    let text = text_from_insert_shortcut(key)?;
    Some(CommandInvocation {
        command: Command::InsertText,
        args: CommandArgs {
            argument: Some(text.to_string()),
            focus_target: CommandFocusTarget::Default,
        },
    })
}

/// Parse a PE-style key name like "c-s", "a-l", "s-tab", or "f2" into an
/// "InputKey". Modifier prefixes "c-"/"a-"/"s-" (Ctrl/Alt/Shift) are
/// case-insensitive and may be combined; the base is a single letter or digit,
/// a named key ("enter", "tab", "up", "pgdn", ...), or "f1"-"f24". Base letters
/// are case-insensitive, so Shift must be written explicitly as "s-". Returns
/// "None" for anything unrecognized.
pub(crate) fn parse_key_name(name: &str) -> Option<InputKey> {
    let mut rest = name.trim();
    if rest.is_empty() {
        return None;
    }

    let mut modifiers = kbmod::NONE;
    while let Some((prefix, tail)) = rest.split_at_checked(2) {
        let modifier = match prefix.to_ascii_lowercase().as_str() {
            "c-" => kbmod::CTRL,
            "a-" => kbmod::ALT,
            "s-" => kbmod::SHIFT,
            _ => break,
        };
        modifiers |= modifier;
        rest = tail;
    }

    if modifiers == kbmod::CTRL && rest.eq_ignore_ascii_case("space") {
        return Some(vk::NULL);
    }

    // Every key is bindable, including the CJK insert keys: a user binding runs
    // before the insert path, so it overrides, and the insert stays as fallback.
    Some(modifiers | base_key(rest)?)
}

fn base_key(name: &str) -> Option<InputKey> {
    let lower = name.to_ascii_lowercase();

    // A single letter, digit, or space maps directly (from_ascii's domain).
    let mut chars = lower.chars();
    if let (Some(ch), None) = (chars.next(), chars.next())
        && let Some(key) = InputKey::from_ascii(ch)
    {
        return Some(key);
    }

    const FUNCTION_KEYS: [InputKey; 24] = [
        vk::F1,
        vk::F2,
        vk::F3,
        vk::F4,
        vk::F5,
        vk::F6,
        vk::F7,
        vk::F8,
        vk::F9,
        vk::F10,
        vk::F11,
        vk::F12,
        vk::F13,
        vk::F14,
        vk::F15,
        vk::F16,
        vk::F17,
        vk::F18,
        vk::F19,
        vk::F20,
        vk::F21,
        vk::F22,
        vk::F23,
        vk::F24,
    ];
    if let Some(n) = lower.strip_prefix('f').and_then(|n| n.parse::<usize>().ok())
        && let Some(key) = n.checked_sub(1).and_then(|idx| FUNCTION_KEYS.get(idx))
    {
        return Some(*key);
    }

    Some(match lower.as_str() {
        "enter" | "return" => vk::RETURN,
        "esc" | "escape" => vk::ESCAPE,
        "tab" => vk::TAB,
        "space" => vk::SPACE,
        "backspace" | "back" | "bksp" => vk::BACK,
        "up" => vk::UP,
        "down" => vk::DOWN,
        "left" => vk::LEFT,
        "right" => vk::RIGHT,
        "home" => vk::HOME,
        "end" => vk::END,
        "pgup" | "pageup" | "prior" => vk::PRIOR,
        "pgdn" | "pagedown" | "next" => vk::NEXT,
        "ins" | "insert" => vk::INSERT,
        "del" | "delete" => vk::DELETE,
        // Named aliases for punctuation keys, which cannot be written literally
        // because the bind grammar reserves them (and some are dead keys).
        "comma" => vk::COMMA,
        "period" | "dot" => vk::PERIOD,
        "colon" => vk::COLON,
        "semicolon" => vk::SEMICOLON,
        "slash" => vk::SLASH,
        "question" => vk::QUESTION,
        "exclamation" | "bang" => vk::EXCLAMATION,
        "apostrophe" | "quote" => vk::APOSTROPHE,
        "lbracket" => vk::LBRACKET,
        "rbracket" => vk::RBRACKET,
        "lbrace" => vk::LBRACE,
        "rbrace" => vk::RBRACE,
        "lt" | "less" => vk::LT,
        "gt" | "greater" => vk::GT,
        _ => return None,
    })
}

pub fn commandbar_shortcut_from_key(key: InputKey) -> Option<CommandBarShortcut> {
    Some(CommandBarShortcut {
        text: match key {
            k if k == vk::F2 => "save ",
            k if k == vk::F3 => "file ",
            k if k == vk::F4 => "quit ",
            _ => return None,
        },
    })
}

pub fn should_handle_command_shortcut_before_editor(command: Command) -> bool {
    matches!(command, Command::InsertText)
}

fn text_from_insert_shortcut(key: InputKey) -> Option<&'static str> {
    INSERT_SHORTCUTS
        .iter()
        .find(|shortcut| shortcut.modifiers | shortcut.key == key)
        .map(|shortcut| shortcut.text)
}

const INSERT_SHORTCUTS: &[InsertShortcut] = &[
    InsertShortcut { modifiers: kbmod::ALT, key: vk::COMMA, text: "，" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::PERIOD, text: "。" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::LT, text: "〈" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::GT, text: "〉" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::SEMICOLON, text: "；" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::COLON, text: "：" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::SEMICOLON, text: "：" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::APOSTROPHE, text: "、" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::LBRACKET, text: "「" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::RBRACKET, text: "」" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::LBRACE, text: "『" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::LBRACKET, text: "『" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::RBRACE, text: "』" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::RBRACKET, text: "』" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N1, text: "！" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N3, text: "△" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N4, text: "□" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N5, text: "☆" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N6, text: "◇" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N7, text: "○" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N8, text: "※" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N9, text: "（" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::N0, text: "）" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::EXCLAMATION, text: "！" },
    InsertShortcut { modifiers: kbmod::ALT_SHIFT, key: vk::N1, text: "！" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::SLASH, text: "？" },
    InsertShortcut { modifiers: kbmod::ALT, key: vk::QUESTION, text: "？" },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_shortcuts_map_to_text_invocations() {
        for (key, expected) in [
            (kbmod::ALT | vk::COMMA, "，"),
            (kbmod::ALT | vk::PERIOD, "。"),
            (kbmod::ALT | vk::DELETE, "。"),
            (kbmod::ALT | vk::SEMICOLON, "；"),
            (kbmod::ALT | vk::COLON, "："),
            (kbmod::ALT | vk::APOSTROPHE, "、"),
            (kbmod::ALT | vk::LBRACKET, "「"),
            (kbmod::ALT | vk::RBRACKET, "」"),
            (kbmod::ALT | vk::LBRACE, "『"),
            (kbmod::ALT | vk::RBRACE, "』"),
            (kbmod::ALT | vk::N1, "！"),
            (kbmod::ALT | vk::EXCLAMATION, "！"),
        ] {
            let Some(CommandInvocation {
                command: Command::InsertText,
                args: CommandArgs { argument: Some(text), .. },
            }) = command_invocation_from_shortcut(key)
            else {
                panic!("insert shortcut did not parse");
            };

            assert!(text == expected);
        }
    }
}
