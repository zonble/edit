// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::tui::Context;

use super::parse::{command_sequence_from_text, macro_name_and_body};
use super::shortcuts::parse_key_name;
use super::{
    Command, CommandArgs, CommandDefinition, CommandFocusTarget, CommandInvocation,
    command_from_text_with_modes, execute_command_sequence,
};
use crate::state::*;

// A macro invoking a macro is just a "RunMacro" step, so recursion is possible.
// Cap the nesting depth; 32 is far deeper than any hand-written macro and stops
// "define a = [macro a]" from looping forever.
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
    CommandDefinition {
        command: Command::Source,
        names: &["source", "load-profile"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: source,
        argument_hint: Some("<path>"),
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

// "bind <key> = [cmd]..." binds a key to a command sequence, the same shape as
// "define" but keyed by a physical key. An empty body removes the binding.
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

    // Clone the sequence out so we no longer borrow "state.macros" while each
    // step takes "&mut state".
    let Some(sequence) = state.macros.get(name).cloned() else {
        return report_error(state, &format!("unknown macro '{name}'"));
    };

    run_capped_sequence(ctx, state, sequence);
}

// "execute" runs the current selection (or the current line) as a command
// sequence: PE's "[execute]", a macro scratchpad in the buffer itself.
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

// Run a sequence under the recursion cap, so a macro or "[execute]" that calls
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

// "source <path>" loads a profile file: a list of commands, one per line, run
// as if typed. This is also the startup loader (see "source_profile_file").
fn source(ctx: &mut Context, state: &mut State, args: CommandArgs) {
    let Some(path) = args.argument.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return report_error(state, "usage: source <path>");
    };
    source_profile_file(ctx, state, path);
}

/// Read a profile file and run its commands. Reused by the "source" command and
/// by startup loading of the "ZE2_PROFILE" file.
pub(crate) fn source_profile_file(ctx: &mut Context, state: &mut State, path: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (ctx, path);
        report_error(state, "loading profile files is not supported on this platform");
    }
    #[cfg(not(target_arch = "wasm32"))]
    match std::fs::read_to_string(path) {
        Ok(text) => run_profile_text(ctx, state, &text),
        Err(err) => report_error(state, &format!("cannot read profile '{path}': {err}")),
    }
}

// Run each command line of a profile as if typed in the command bar. A line
// that fails to parse or aborts at runtime is counted but does not stop the
// rest of the profile, so one bad line does not sink it. If any line failed,
// the abort flag is set at the end so an enclosing "source" step stops too.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn run_profile_text(ctx: &mut Context, state: &mut State, text: &str) {
    // Count profile lines as nested execution so a profile that sources itself
    // is bounded by the same depth cap as macros.
    if state.macro_depth >= MAX_MACRO_DEPTH {
        return report_error(state, "profile nesting too deep");
    }
    state.macro_depth += 1;

    let include_vim = state.command_bar_include_vim_commands;
    let include_emacs = state.command_bar_include_emacs_commands;

    let mut failures = 0usize;
    let mut first_failure = None;
    for (line_no, command) in profile_command_lines(text) {
        let parsed =
            command_sequence_from_text(&command, include_vim, include_emacs).or_else(|| {
                command_from_text_with_modes(&command, include_vim, include_emacs)
                    .map(|inv| vec![inv])
            });

        let failed = match parsed {
            Some(sequence) => {
                state.macro_abort = false;
                execute_command_sequence(ctx, state, sequence);
                state.macro_abort
            }
            None => true,
        };
        if failed {
            failures += 1;
            first_failure.get_or_insert(line_no);
        }
    }

    state.macro_depth -= 1;

    // Start clean, then flag an abort only if a line failed, so the summary
    // propagates to any enclosing sequence.
    state.macro_abort = false;
    if let Some(first) = first_failure {
        report_error(state, &format!("profile: {failures} line(s) failed (first: line {first})"));
    }
}

// Turn profile text into runnable command lines, paired with the 1-based line
// they start on. Blank and "#" comment lines are dropped, even in the middle of
// a continuation; a line ending in "\" continues onto the next.
fn profile_command_lines(text: &str) -> Vec<(usize, String)> {
    let mut lines = Vec::new();
    let mut pending: Option<(usize, String)> = None;

    for (idx, raw) in text.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let (start, mut acc) = match pending.take() {
            Some((start, mut acc)) => {
                acc.push_str(raw);
                (start, acc)
            }
            None => (idx + 1, raw.to_string()),
        };

        match acc.strip_suffix('\\') {
            Some(head) => {
                acc.truncate(head.len());
                pending = Some((start, acc));
            }
            None => lines.push((start, acc)),
        }
    }

    // An unterminated continuation at end of file: keep the "\" so the line
    // fails to parse rather than running a truncated command.
    if let Some((start, mut acc)) = pending {
        acc.push('\\');
        lines.push((start, acc));
    }

    lines
}

// Handlers cannot return a Result, so surface failures in the command bar the
// same way the built-in query/help commands do, and flag an abort so any
// enclosing macro stops instead of finishing its remaining steps.
fn report_error(state: &mut State, message: &str) {
    state.command_bar_error = message.to_string();
    state.command_bar_active = true;
    state.macro_abort = true;
}

#[cfg(test)]
mod tests {
    use super::profile_command_lines;

    #[test]
    fn profile_lines_drop_comments_and_join_continuations() {
        // Blank and "#" lines vanish, even between a "\" and its continuation.
        // Each command keeps the 1-based number of the line it starts on.
        let text =
            "# header\n\nsave\ndefine x = \\\n  # mid comment\n[undo] [redo]\n  # note\nquit";
        let lines = profile_command_lines(text);
        assert_eq!(
            lines,
            vec![
                (3, "save".to_string()),
                (4, "define x = [undo] [redo]".to_string()),
                (8, "quit".to_string()),
            ]
        );
    }

    #[test]
    fn profile_dangling_continuation_keeps_backslash() {
        // A trailing "\" with no next line stays malformed so it will not run.
        assert_eq!(profile_command_lines("[delete] \\"), vec![(1, "[delete] \\".to_string())]);
    }
}
