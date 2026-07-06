// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::input::vk;
use ze2::tui::Context;

use crate::commands::{
    Command, CommandInvocation, command_invocation_from_shortcut, commandbar_shortcut_from_key,
    execute_command_sequence, should_handle_command_shortcut_before_editor,
};
use crate::settings::BindingMode;
use crate::state::{State, StateSearchKind};

pub fn handle_input_before_editor(
    ctx: &mut Context,
    state: &mut State,
) -> Option<CommandInvocation> {
    if handle_commandbar_shortcut(ctx, state) {
        ctx.set_input_consumed();
        return None;
    }

    insert_text_invocation_before_editor(ctx, state)
}

// Run a user binding before anything else in the frame (called at the top of
// draw, before the menubar), so it can override any key: text-area editing keys,
// the menu accelerators (Alt+letter), and the default profile alike.
pub(crate) fn run_user_binding_before_editor(ctx: &mut Context, state: &mut State) -> bool {
    // Only in the editor: never intercept keys meant for an open menu, the
    // command bar, the search field, a dialog, or a pending clipboard sync. (A
    // binding still overrides a menu accelerator while the menu is closed, since
    // this runs before the menubar claims the key.)
    if state.menubar_visible
        || state.command_bar_active
        || !matches!(state.wants_search.kind, StateSearchKind::Hidden | StateSearchKind::Disabled)
        || state.wants_dialog()
        || ctx.clipboard_ref().wants_host_sync()
    {
        return false;
    }

    let Some(key) = ctx.keyboard_input() else {
        return false;
    };
    let Some(sequence) = user_binding_for_key(state, key) else {
        return false;
    };

    state.macro_abort = false;
    execute_command_sequence(ctx, state, sequence);
    true
}

fn user_binding_for_key(
    state: &State,
    key: ze2::input::InputKey,
) -> Option<Vec<CommandInvocation>> {
    state.user_bindings.get(&key).cloned()
}

fn handle_commandbar_shortcut(ctx: &Context, state: &mut State) -> bool {
    if close_commandbar_before_editor(ctx, state) || focus_commandbar_before_editor(ctx, state) {
        return true;
    }

    false
}

fn close_commandbar_before_editor(ctx: &Context, state: &mut State) -> bool {
    if !state.command_bar_active
        || ctx.keyboard_input() != Some(vk::ESCAPE)
        || state.wants_dialog()
        || ctx.clipboard_ref().wants_host_sync()
    {
        return false;
    }

    state.command_bar_active = false;
    state.command_bar_focus = false;
    state.command_bar_error.clear();
    state.wants_editor_focus = true;
    true
}

fn focus_commandbar_before_editor(ctx: &Context, state: &mut State) -> bool {
    if !matches!(state.wants_search.kind, StateSearchKind::Hidden | StateSearchKind::Disabled)
        || state.wants_dialog()
        || ctx.clipboard_ref().wants_host_sync()
    {
        return false;
    }

    if ctx.keyboard_input() == Some(vk::ESCAPE) {
        state.command_bar_focus = true;
        return true;
    }

    let Some(shortcut) = ctx.keyboard_input().and_then(commandbar_shortcut_from_key) else {
        return false;
    };

    state.command_bar_input.clear();
    state.command_bar_input.push_str(shortcut.text);
    state.command_bar_error.clear();
    state.command_bar_active = true;
    state.command_bar_focus = true;
    true
}

fn insert_text_invocation_before_editor(
    ctx: &Context,
    state: &mut State,
) -> Option<CommandInvocation> {
    if state.command_bar_active
        || !matches!(state.wants_search.kind, StateSearchKind::Hidden | StateSearchKind::Disabled)
        || state.wants_dialog()
        || ctx.clipboard_ref().wants_host_sync()
    {
        return None;
    }

    let keyboard_input = ctx.keyboard_input()?;

    if cfg!(any(target_os = "macos", target_os = "ios")) && state.binding == BindingMode::Ghostty {
        if keyboard_input == crate::input::kbmod::CTRL | vk::A {
            return Some(CommandInvocation {
                command: Command::MoveToVisualLineBegin,
                args: Default::default(),
            });
        }
        if keyboard_input == crate::input::kbmod::CTRL | vk::E {
            return Some(CommandInvocation {
                command: Command::MoveToVisualLineEnd,
                args: Default::default(),
            });
        }
    }

    let invocation = command_invocation_from_shortcut(keyboard_input)?;
    if should_handle_command_shortcut_before_editor(invocation.command) {
        Some(invocation)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use ze2::input::{kbmod, vk};

    use super::*;

    #[test]
    fn binding_before_editor_uses_user_profile_only() {
        let key = kbmod::ALT | vk::B;
        let mut state = State::new().unwrap();
        state.key_bindings.insert(
            key,
            vec![CommandInvocation { command: Command::MarkBlock, args: Default::default() }],
        );

        assert!(user_binding_for_key(&state, key).is_none());

        state.user_bindings.insert(
            key,
            vec![CommandInvocation { command: Command::MoveMark, args: Default::default() }],
        );

        let sequence = user_binding_for_key(&state, key).unwrap();
        assert!(sequence[0].command == Command::MoveMark);
    }
}
