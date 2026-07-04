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
pub(crate) use macro_commands::source_profile_file;
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

// Cap the record buffer so an unattended "record on" cannot grow without bound.
const MAX_RECORDED: usize = 4096;

pub fn execute_command_invocation(
    ctx: &mut Context,
    state: &mut State,
    invocation: CommandInvocation,
) {
    // Record top-level invocations while recording. Never record a replay (the
    // replaying guard) or the record/replay controls themselves, and skip steps
    // inside a macro (depth > 0) since replaying the macro call re-runs them.
    if state.recording
        && !state.replaying
        && state.macro_depth == 0
        && state.recorded.len() < MAX_RECORDED
        && !matches!(invocation.command, Command::RecordToggle | Command::Replay)
    {
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
