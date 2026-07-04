// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use super::parse::{command_sequence_from_text, macro_name_and_body};
use super::shortcuts::parse_key_name;
use super::{
    Command, CommandArgs, CommandDefinition, CommandFocusTarget, CommandInvocation,
    execute_command_sequence,
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
    CommandDefinition {
        command: Command::BindKey,
        names: &["bind"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: bind_key,
        argument_hint: Some("<key> = [cmd]..."),
    },
    CommandDefinition {
        command: Command::ExecuteRegion,
        names: &["execute", "run-region"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: execute_region,
        argument_hint: None,
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

// `bind <key> = [cmd]...` binds a key to a command sequence, the same shape as
// `define` but keyed by a physical key. An empty body removes the binding.
fn bind_key(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let Some(arg) = args.argument.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return report_error(state, "usage: bind <key> = [cmd] [cmd]...");
    };
    let Some((key_name, body)) = macro_name_and_body(arg) else {
        return report_error(state, "bind needs '<key> = [cmd]...' with a single key name");
    };
    let Some(key) = parse_key_name(key_name) else {
        return report_error(state, &format!("unknown key '{key_name}'"));
    };

    if body.is_empty() {
        state.key_bindings.remove(&key);
        return;
    }

    let Some(sequence) = command_sequence_from_text(
        body,
        state.command_bar_include_vim_commands,
        state.command_bar_include_emacs_commands,
    ) else {
        return report_error(state, "binding body must be a command sequence: [cmd] [cmd]...");
    };

    state.key_bindings.insert(key, sequence);
}

fn run_macro(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let Some(name) = args.argument.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return report_error(state, "usage: macro <name>");
    };

    // Clone the sequence out so we no longer borrow `state.macros` while each
    // step takes `&mut state`.
    let Some(sequence) = state.macros.get(name).cloned() else {
        return report_error(state, &format!("unknown macro '{name}'"));
    };

    run_capped_sequence(ctx, state, sequence);
}

// `execute` runs the current selection (or the current line) as a command
// sequence -- PE's `[execute]`, a macro scratchpad in the buffer itself.
fn execute_region(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let bytes = {
        let Some(doc) = state.documents.active() else {
            return report_error(state, "no active document");
        };
        let mut buffer = doc.buffer.borrow_mut();
        buffer.extract_user_selection(false).unwrap_or_else(|| buffer.current_line_text())
    };

    let Ok(text) = String::from_utf8(bytes) else {
        return report_error(state, "region is not valid UTF-8");
    };
    let Some(sequence) = command_sequence_from_text(
        text.trim(),
        state.command_bar_include_vim_commands,
        state.command_bar_include_emacs_commands,
    ) else {
        return report_error(state, "region is not a command sequence: [cmd] [cmd]...");
    };

    run_capped_sequence(ctx, state, sequence);
}

// Run a sequence under the recursion cap, so a macro or `[execute]` that calls
// itself stops instead of overflowing the stack. Resets the abort flag on a
// fresh top-level run, and relies on execute_command_sequence to stop the rest
// of the sequence once a step aborts.
fn run_capped_sequence(ctx: &mut Context, state: &mut State, sequence: Vec<CommandInvocation>) {
    if state.macro_depth == 0 {
        state.macro_abort = false;
    }
    if state.macro_depth >= MAX_MACRO_DEPTH {
        return report_error(state, "macro recursion too deep");
    }

    state.macro_depth += 1;
    execute_command_sequence(ctx, state, sequence);
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
