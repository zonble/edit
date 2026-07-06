// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use crate::state::State;

mod arguments;
mod definition;
mod editing_commands;
mod file_commands;
mod file_format_commands;
mod macro_commands;
mod navigation_commands;
mod parse;
mod search_commands;
mod settings_commands;
mod shortcuts;
mod utility_commands;
mod view_commands;

pub use definition::{
    Command, CommandArgs, CommandBarShortcut, CommandFocusTarget, CommandInvocation,
};
pub(crate) use macro_commands::{load_default_profile, source_profile_file};
pub use parse::{
    autocomplete_command_suggestions_with_modes, command_from_text_with_modes,
    command_sequence_from_text,
};
pub use shortcuts::{
    command_invocation_from_shortcut, commandbar_shortcut_from_key,
    should_handle_command_shortcut_before_editor,
};

use definition::CommandDefinition;

const COMMAND_GROUPS: &[&[CommandDefinition]] = &[
    file_commands::COMMANDS,
    file_format_commands::COMMANDS,
    editing_commands::COMMANDS,
    search_commands::COMMANDS,
    navigation_commands::COMMANDS,
    view_commands::COMMANDS,
    settings_commands::COMMANDS,
    utility_commands::COMMANDS,
    macro_commands::COMMANDS,
];

pub(crate) fn command_definitions() -> impl Iterator<Item = &'static CommandDefinition> {
    COMMAND_GROUPS
        .iter()
        .flat_map(|group| group.iter())
        .filter(|definition| command_visible_in_current_target(definition.command))
}

fn command_definition(command: Command) -> Option<&'static CommandDefinition> {
    command_definitions().find(|definition| definition.command == command)
}

fn command_visible_in_current_target(command: Command) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        !matches!(
            command,
            Command::TransformUppercase
                | Command::TransformLowercase
                | Command::TransformHalfWidth
                | Command::TransformFullWidth
                | Command::TransformLatin
                | Command::TransformKatakana
                | Command::TransformHiragana
                | Command::TransformSimplifiedChinese
                | Command::TransformTraditionalChinese
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = command;
        true
    }
}

pub fn execute_command(ctx: &mut Context, state: &mut State, command: Command) {
    execute_command_invocation(
        ctx,
        state,
        CommandInvocation { command, args: CommandArgs::default() },
    );
}

pub fn execute_command_invocation(
    ctx: &mut Context,
    state: &mut State,
    invocation: CommandInvocation,
) {
    // Record top-level invocations while recording (see macro_commands::should_record).
    if macro_commands::should_record(state, invocation.command, false) {
        state.recorded.push(invocation.clone());
    }

    let Some(definition) = command_definition(invocation.command) else {
        return;
    };

    (definition.handler)(ctx, state, invocation.args);

    ctx.needs_rerender();
}

/// Run a sequence of invocations in order, stopping early if a step aborts the
/// enclosing macro (see "State::macro_abort"). Top-level callers clear
/// "macro_abort" before calling so a prior failure does not leak in.
pub fn execute_command_sequence(
    ctx: &mut Context,
    state: &mut State,
    sequence: Vec<CommandInvocation>,
) {
    for invocation in sequence {
        execute_command_invocation(ctx, state, invocation);
        if state.macro_abort {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use ze2::buffer::{CursorMovement, TextBuffer};
    use ze2::helpers::Point;
    use ze2::tui::Tui;

    use super::{Command, execute_command};
    use crate::state::State;

    // Build a scratch buffer with `text` and the cursor at byte `offset`, run
    // `op` on it, and return its (contents, cursor offset).
    fn after_op(text: &[u8], offset: usize, op: fn(&mut TextBuffer)) -> (String, usize) {
        let mut buf = TextBuffer::new(false).unwrap();
        buf.set_crlf(false);
        buf.set_insert_final_newline(false);
        buf.write_raw(text);
        buf.cursor_move_to_offset(offset);
        op(&mut buf);
        let mut contents = String::new();
        buf.save_as_string(&mut contents);
        (contents, buf.cursor_offset())
    }

    // Run `command` through the real dispatch on a scratch document and return its
    // (contents, cursor offset).
    fn after_command(text: &[u8], offset: usize, command: Command) -> (String, usize) {
        let mut tui = Tui::new().unwrap();
        let mut ctx = tui.create_context(None);
        let mut state = State::new().unwrap();
        state.documents.add_untitled().unwrap();
        {
            let mut buf = state.documents.active().unwrap().buffer.borrow_mut();
            buf.set_crlf(false);
            buf.set_insert_final_newline(false);
            buf.write_raw(text);
            buf.cursor_move_to_offset(offset);
        }
        execute_command(&mut ctx, &mut state, command);
        let mut buf = state.documents.active().unwrap().buffer.borrow_mut();
        let mut contents = String::new();
        buf.save_as_string(&mut contents);
        (contents, buf.cursor_offset())
    }

    // The default profile binds these keys to commands that shadow the text area;
    // each command must apply the same buffer mutation as the op tui.rs runs for
    // that key (no selection), or the profile silently drifts from the built-in
    // behavior. The right column is that text-area op, transcribed from tui.rs.
    // A buffer mutation the text area runs for a key, e.g. delete one grapheme.
    type TextAreaOp = fn(&mut TextBuffer);

    #[test]
    fn editor_commands_match_their_text_area_ops() {
        let text = b"alpha beta\ngamma delta";
        let offset = 6; // start of "beta"
        let cases: &[(Command, TextAreaOp)] = &[
            (Command::MoveLeft, |b| b.cursor_move_delta(CursorMovement::Grapheme, -1)),
            (Command::MoveRight, |b| b.cursor_move_delta(CursorMovement::Grapheme, 1)),
            (Command::MoveToWordBegin, |b| b.cursor_move_delta(CursorMovement::Word, -1)),
            (Command::MoveToWordEnd, |b| b.cursor_move_delta(CursorMovement::Word, 1)),
            (Command::MoveToDocumentBegin, |b| b.cursor_move_to_visual(Point::default())),
            (Command::MoveToDocumentEnd, |b| b.cursor_move_to_visual(Point::MAX)),
            (Command::DeleteForward, |b| b.delete(CursorMovement::Grapheme, 1)),
            (Command::DeleteBackward, |b| b.delete(CursorMovement::Grapheme, -1)),
            (Command::DeleteWordForward, |b| b.delete(CursorMovement::Word, 1)),
            (Command::DeleteWordBackward, |b| b.delete(CursorMovement::Word, -1)),
            (Command::DeleteLine, |b| b.delete_line()),
        ];
        for (i, &(command, op)) in cases.iter().enumerate() {
            assert_eq!(
                after_command(text, offset, command),
                after_op(text, offset, op),
                "case {i} must match its text-area op",
            );
        }
    }
}
