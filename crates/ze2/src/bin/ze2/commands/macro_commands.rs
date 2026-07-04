// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use super::parse::{command_sequence_from_text, macro_name_and_body};
use super::{
    Command, CommandArgs, CommandDefinition, CommandFocusTarget, execute_command_invocation,
};
use crate::state::*;

// A macro invoking a macro is just a `RunMacro` step, so recursion is possible.
// Cap the nesting depth; 32 is far deeper than any hand-written macro and stops
// `define a = [macro a]` from looping forever.
const MAX_MACRO_DEPTH: usize = 32;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::DefineMacro,
        names: &["define"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: define_macro,
        argument_hint: Some("<name> = [cmd]..."),
    },
    CommandDefinition {
        command: Command::RunMacro,
        names: &["macro", "run-macro"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: run_macro,
        argument_hint: Some("<name>"),
    },
];

fn define_macro(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let Some(arg) = args.argument.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return report_error(state, "usage: define <name> = [cmd] [cmd]...");
    };
    let Some((name, body)) = macro_name_and_body(arg) else {
        return report_error(state, "define needs '<name> = [cmd]...' with a single-word name");
    };

    // Empty body removes the macro (PE-style unbind).
    if body.is_empty() {
        state.macros.remove(name);
        return;
    }

    let Some(sequence) = command_sequence_from_text(
        body,
        state.command_bar_include_vim_commands,
        state.command_bar_include_emacs_commands,
    ) else {
        return report_error(state, "macro body must be a command sequence: [cmd] [cmd]...");
    };

    state.macros.insert(name.to_string(), sequence);
}

fn run_macro(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    // A fresh top-level invocation starts with no pending abort.
    if state.macro_depth == 0 {
        state.macro_abort = false;
    }

    let Some(name) = args.argument.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return report_error(state, "usage: macro <name>");
    };

    // Clone the sequence out so we no longer borrow `state.macros` while each
    // step takes `&mut state`.
    let Some(sequence) = state.macros.get(name).cloned() else {
        return report_error(state, &format!("unknown macro '{name}'"));
    };

    if state.macro_depth >= MAX_MACRO_DEPTH {
        return report_error(state, "macro recursion too deep");
    }

    state.macro_depth += 1;
    for invocation in sequence {
        execute_command_invocation(ctx, state, invocation);
        // A failed step (e.g. the depth cap) aborts the whole sequence so the
        // remaining steps do not run once per unwound frame.
        if state.macro_abort {
            break;
        }
    }
    state.macro_depth -= 1;
}

// Handlers cannot return a Result, so surface failures in the command bar the
// same way the built-in query/help commands do, and flag an abort so any
// enclosing macro stops instead of finishing its remaining steps.
fn report_error(state: &mut State, message: &str) {
    state.command_bar_error = message.to_string();
    state.command_bar_active = true;
    state.macro_abort = true;
}
