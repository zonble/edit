// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::input::{InputKey, InputKeyMod, kbmod, vk};

use super::{Command, CommandArgs, CommandBarShortcut, CommandFocusTarget, CommandInvocation};

struct InsertShortcut {
    modifiers: InputKeyMod,
    key: InputKey,
    text: &'static str,
}

pub fn command_invocation_from_shortcut(key: InputKey) -> Option<CommandInvocation> {
    if let Some(text) = text_from_insert_shortcut(key) {
        return Some(CommandInvocation {
            command: Command::InsertText,
            args: CommandArgs {
                argument: Some(text.to_string()),
                focus_target: CommandFocusTarget::Default,
            },
        });
    }

    Some(match key {
        k if k == kbmod::ALT | vk::B => Command::MarkBlock,
        k if k == kbmod::ALT | vk::L => Command::MarkLine,
        k if k == kbmod::ALT | vk::M => Command::MoveMark,
        k if k == kbmod::ALT | vk::C => Command::CopyMark,
        k if k == kbmod::ALT | vk::F => Command::FillMark,
        k if k == kbmod::ALT | vk::U => Command::Unmark,
        k if k == kbmod::CTRL | vk::N => Command::NewFile,
        k if k == kbmod::CTRL | vk::O => Command::OpenFile,
        k if k == kbmod::CTRL | vk::S => Command::Save,
        k if k == kbmod::CTRL_SHIFT | vk::S => Command::SaveAs,
        k if k == kbmod::CTRL | vk::W => Command::CloseFile,
        k if k == kbmod::CTRL | vk::P => Command::GoToFile,
        k if k == kbmod::CTRL | vk::Q => Command::Exit,
        k if k == kbmod::CTRL | vk::G => Command::Goto,
        k if k == kbmod::CTRL | vk::F => Command::Find,
        k if k == kbmod::CTRL | vk::R => Command::Replace,
        k if k == kbmod::CTRL | vk::L => Command::SelectLine,
        _ => return None,
    })
    .map(|command| CommandInvocation { command, args: CommandArgs::default() })
}

/// Parse a PE-style key name like "c-s", "a-l", "s-tab", or "f2" into an
/// "InputKey". Modifier prefixes "c-"/"a-"/"s-" (Ctrl/Alt/Shift) are
/// case-insensitive and may be combined; the base is a single letter or digit,
/// a named key ("enter", "tab", "up", "pgdn", ...), or "f1"-"f12". Base letters
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

    Some(match lower.as_str() {
        "f1" => vk::F1,
        "f2" => vk::F2,
        "f3" => vk::F3,
        "f4" => vk::F4,
        "f5" => vk::F5,
        "f6" => vk::F6,
        "f7" => vk::F7,
        "f8" => vk::F8,
        "f9" => vk::F9,
        "f10" => vk::F10,
        "f11" => vk::F11,
        "f12" => vk::F12,
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

    #[test]
    fn key_names_parse_with_modifiers() {
        assert!(parse_key_name("c-s") == Some(kbmod::CTRL | vk::S));
        assert!(parse_key_name("a-l") == Some(kbmod::ALT | vk::L));
        assert!(parse_key_name("s-tab") == Some(kbmod::SHIFT | vk::TAB));
        // Modifiers combine and are case-insensitive.
        assert!(parse_key_name("C-S-f2") == Some(kbmod::CTRL_SHIFT | vk::F2));
        // Bare keys and named keys.
        assert!(parse_key_name("a") == Some(vk::A));
        assert!(parse_key_name("enter") == Some(vk::RETURN));
        assert!(parse_key_name("f12") == Some(vk::F12));
        // Junk is rejected.
        assert!(parse_key_name("").is_none());
        assert!(parse_key_name("c-").is_none());
        assert!(parse_key_name("nope").is_none());
    }

    #[test]
    fn mark_shortcuts_map_to_commands() {
        for (key, expected) in [
            (kbmod::ALT | vk::B, Command::MarkBlock),
            (kbmod::ALT | vk::L, Command::MarkLine),
            (kbmod::ALT | vk::M, Command::MoveMark),
            (kbmod::ALT | vk::C, Command::CopyMark),
        ] {
            let Some(CommandInvocation { command, args }) = command_invocation_from_shortcut(key)
            else {
                panic!("mark shortcut did not parse");
            };

            assert!(command == expected);
            assert!(args.argument.is_none());
        }
    }
}
