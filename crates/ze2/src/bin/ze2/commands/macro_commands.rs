// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::input::kbmod;
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

// Cap the record buffer so an unattended "record on" cannot grow without bound.
pub(crate) const MAX_RECORDED: usize = 4096;

// Whether a top-level invocation should be appended to the recording. Shared by
// the native and web command runners so the record/replay rules live in one
// place. Never record a replay or the record/replay controls themselves, and
// skip steps inside a macro (depth > 0) since replaying the macro re-runs them.
// "exclude_async" drops a command a replay cannot re-issue in order: Paste on the
// web completes through an async host round-trip.
pub(crate) fn should_record(state: &State, command: Command, exclude_async: bool) -> bool {
    state.recording
        && !state.replaying
        && state.macro_depth == 0
        && state.recorded.len() < MAX_RECORDED
        && !matches!(command, Command::RecordToggle | Command::Replay)
        && !(exclude_async && command == Command::Paste)
}

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
    CommandDefinition {
        command: Command::RecordToggle,
        names: &["record"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: record_toggle,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Replay,
        names: &["replay", "play-macro"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: replay,
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
        state.user_bindings.remove(&key);
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

    if !state.loading_default_profile || !is_noop_sequence(&sequence) {
        state.user_bindings.insert(key, sequence.clone());
    }
    state.key_bindings.insert(key, sequence);

    // On macOS an Alt (Option) key often does not reach ze2 as Alt unless the
    // terminal is set to send Option as Meta; it is a dead key or types a
    // special character otherwise. Warn only for an interactive bind (depth 0),
    // not while a profile file is loading, and only for keys carrying Alt.
    if state.macro_depth == 0
        && cfg!(any(target_os = "macos", target_os = "ios"))
        && key.modifiers_contains(kbmod::ALT)
    {
        report_status(state, "bound; macOS Alt keys need the terminal to send Option as Meta");
    }
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

// The default profile is compiled into the binary from assets/ze2.pro, so ze2 has
// a set of macros and bindings even with no profile file on disk.
const DEFAULT_PROFILE: &str = include_str!("../../../../../../assets/ze2.pro");

/// Load the compiled-in default profile. Run at startup before any user profile,
/// so a user profile can override what it defines.
pub(crate) fn load_default_profile(ctx: &mut Context, state: &mut State) {
    state.loading_default_profile = true;
    run_profile_text(ctx, state, DEFAULT_PROFILE);
    state.loading_default_profile = false;
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

struct ProfileSnapshot {
    macros: std::collections::HashMap<String, Vec<CommandInvocation>>,
    key_bindings: std::collections::HashMap<ze2::input::InputKey, Vec<CommandInvocation>>,
    user_bindings: std::collections::HashMap<ze2::input::InputKey, Vec<CommandInvocation>>,
}

type ParsedProfileCommand = (usize, String, Vec<CommandInvocation>);
type ProfileParseError = (usize, usize);

impl ProfileSnapshot {
    fn capture(state: &State) -> Self {
        Self {
            macros: state.macros.clone(),
            key_bindings: state.key_bindings.clone(),
            user_bindings: state.user_bindings.clone(),
        }
    }

    fn restore(self, state: &mut State) {
        state.macros = self.macros;
        state.key_bindings = self.key_bindings;
        state.user_bindings = self.user_bindings;
    }
}

fn parse_profile_commands(
    text: &str,
    include_vim: bool,
    include_emacs: bool,
) -> Result<Vec<ParsedProfileCommand>, ProfileParseError> {
    let mut parsed = Vec::new();
    let mut failures = 0usize;
    let mut first_failure = None;

    for (line_no, command) in profile_command_lines(text) {
        if let Some(sequence) = parse_profile_command(&command, include_vim, include_emacs) {
            parsed.push((line_no, command, sequence));
        } else {
            failures += 1;
            first_failure.get_or_insert(line_no);
        }
    }

    match first_failure {
        Some(first) => Err((failures, first)),
        None => Ok(parsed),
    }
}

fn is_noop_sequence(sequence: &[CommandInvocation]) -> bool {
    matches!(sequence, [invocation] if invocation.command == Command::Noop)
}

fn parse_profile_command(
    command: &str,
    include_vim: bool,
    include_emacs: bool,
) -> Option<Vec<CommandInvocation>> {
    if let Some(rest) = command.strip_prefix("bind ") {
        let (key_name, body) = macro_name_and_body(rest)?;
        parse_key_name(key_name)?;
        if !body.is_empty() {
            command_sequence_from_text(body, include_vim, include_emacs)?;
        }
    } else if let Some(rest) = command.strip_prefix("define ") {
        let (_, body) = macro_name_and_body(rest)?;
        if !body.is_empty() {
            command_sequence_from_text(body, include_vim, include_emacs)?;
        }
    }

    command_sequence_from_text(command, include_vim, include_emacs).or_else(|| {
        command_from_text_with_modes(command, include_vim, include_emacs).map(|inv| vec![inv])
    })
}

// Run each command line of a profile as if typed in the command bar. A line
// that fails to parse aborts the whole profile before any line runs. A runtime
// failure restores the user's macro/binding tables, so a broken profile cannot
// leave a half-applied key map over the compiled default profile.
fn run_profile_text(ctx: &mut Context, state: &mut State, text: &str) {
    // Count profile lines as nested execution: it bounds "source" that sources
    // itself, and keeps the recorder from capturing the loaded commands (only
    // the "source" command is recorded, and replaying it re-loads the file).
    if state.macro_depth >= MAX_MACRO_DEPTH {
        return report_error(state, "profile nesting too deep");
    }
    state.macro_depth += 1;

    let include_vim = state.command_bar_include_vim_commands;
    let include_emacs = state.command_bar_include_emacs_commands;
    let parsed = match parse_profile_commands(text, include_vim, include_emacs) {
        Ok(parsed) => parsed,
        Err((failures, first)) => {
            state.macro_depth -= 1;
            return report_error(
                state,
                &format!(
                    "profile: {failures} parse error(s) (first: line {first}); using defaults"
                ),
            );
        }
    };

    let mut failures = 0usize;
    let mut first_failure = None;
    let snapshot = (!state.loading_default_profile).then(|| ProfileSnapshot::capture(state));

    for (line_no, _command, sequence) in parsed {
        state.macro_abort = false;
        execute_command_sequence(ctx, state, sequence);
        let failed = state.macro_abort;
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
        if let Some(snapshot) = snapshot {
            snapshot.restore(state);
        }
        report_error(
            state,
            &format!("profile: {failures} line(s) failed (first: line {first}); using defaults"),
        );
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

// "record" toggles command recording. Starting clears the previous take; each
// top-level command is then appended by execute_command_invocation until stop.
fn record_toggle(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if state.recording {
        state.recording = false;
        report_status(state, &format!("recording stopped ({} steps)", state.recorded.len()));
    } else {
        state.recorded.clear();
        state.recording = true;
        report_status(state, "recording started");
    }
}

// "replay" runs the recorded commands back through the normal execution path.
// The replaying guard keeps the replay itself out of the recorder.
fn replay(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    // A recorded macro/execute/source could expand to "replay"; guard against
    // replay driving itself.
    if state.replaying {
        return report_error(state, "already replaying");
    }
    if state.recording {
        return report_error(state, "stop recording before replay");
    }
    if state.recorded.is_empty() {
        return report_error(state, "nothing recorded to replay");
    }

    let sequence = state.recorded.clone();
    state.replaying = true;
    state.macro_abort = false;
    execute_command_sequence(ctx, state, sequence);
    state.replaying = false;
}

// Show a one-line status in the command bar, the way the query/help commands do.
fn report_status(state: &mut State, message: &str) {
    state.command_bar_error = message.to_string();
    state.command_bar_error_is_warning = true;
    state.command_bar_active = true;
}

// Like report_status, but also flags an abort so any enclosing macro stops
// instead of finishing its remaining steps.
fn report_error(state: &mut State, message: &str) {
    report_status(state, message);
    state.command_bar_error_is_warning = false;
    state.macro_abort = true;
}

// The macro/profile/binding test suite. Only pure parsing and profile logic is
// covered here; running a macro or binding needs a live Context and State, so
// those paths are exercised by hand.
#[cfg(test)]
mod tests {
    use ze2::input::{kbmod, vk};
    use ze2::tui::Tui;

    use super::parse_key_name;
    use super::{
        Command, DEFAULT_PROFILE, command_from_text_with_modes, command_sequence_from_text,
        macro_name_and_body, parse_profile_commands, profile_command_lines, run_profile_text,
    };
    use crate::state::State;

    #[test]
    fn command_sequences_parse_and_validate() {
        let seq = command_sequence_from_text("[undo] [save]", true, true).unwrap();
        assert!(seq.len() == 2);
        assert!(matches!(seq[0].command, Command::Undo));
        assert!(matches!(seq[1].command, Command::Save));
        assert!(command_sequence_from_text("[undo] [not-a-command]", true, true).is_none());
        assert!(command_sequence_from_text("undo", true, true).is_none());

        let seq = command_sequence_from_text("[3 undo]", true, true).unwrap();
        assert!(seq.len() == 3);
        assert!(seq.iter().all(|inv| matches!(inv.command, Command::Undo)));
        let seq = command_sequence_from_text("[2 find foo]", true, true).unwrap();
        assert!(seq.len() == 2);
        assert!(seq.iter().all(|inv| matches!(inv.command, Command::Find)));
        let seq = command_sequence_from_text("[42]", true, true).unwrap();
        assert!(seq.len() == 1 && matches!(seq[0].command, Command::Goto));
        assert!(command_sequence_from_text("[5000 undo]", true, true).is_none());
        // A zero repeat is rejected, not silently expanded to nothing.
        assert!(command_sequence_from_text("[0 undo]", true, true).is_none());
    }

    #[test]
    fn macro_definitions_parse() {
        assert_eq!(macro_name_and_body("greet = [insert hi]"), Some(("greet", "[insert hi]")));
        assert_eq!(macro_name_and_body("greet ="), Some(("greet", "")));
        assert_eq!(macro_name_and_body("two words = [undo]"), None);
        assert_eq!(macro_name_and_body("[weird] = [undo]"), None);
        assert_eq!(macro_name_and_body("nobody"), None);
        assert_eq!(macro_name_and_body("= [undo]"), None);
    }

    #[test]
    fn key_names_parse_for_profile_bindings() {
        assert!(parse_key_name("c-s") == Some(kbmod::CTRL | vk::S));
        assert!(parse_key_name("a-l") == Some(kbmod::ALT | vk::L));
        assert!(parse_key_name("s-tab") == Some(kbmod::SHIFT | vk::TAB));
        assert!(parse_key_name("C-S-f2") == Some(kbmod::CTRL_SHIFT | vk::F2));
        assert!(parse_key_name("a") == Some(vk::A));
        assert!(parse_key_name("enter") == Some(vk::RETURN));
        assert!(parse_key_name("f12") == Some(vk::F12));
        assert!(parse_key_name("c-space") == Some(vk::NULL));
        assert!(parse_key_name("a-comma") == Some(kbmod::ALT | vk::COMMA));
        assert!(parse_key_name("c-slash") == Some(kbmod::CTRL | vk::SLASH));
        assert!(parse_key_name("f24") == Some(vk::F24));
        assert!(parse_key_name("").is_none());
        assert!(parse_key_name("c-").is_none());
        assert!(parse_key_name("nope").is_none());
        assert!(parse_key_name("f25").is_none());

        for name in [
            "backspace",
            "c-backspace",
            "tab",
            "s-tab",
            "enter",
            "f9",
            "esc",
            "pgup",
            "s-pgup",
            "pgdn",
            "s-pgdn",
            "home",
            "c-home",
            "s-home",
            "end",
            "c-end",
            "s-end",
            "left",
            "c-left",
            "s-left",
            "up",
            "c-up",
            "s-up",
            "a-up",
            "right",
            "c-right",
            "s-right",
            "down",
            "c-down",
            "s-down",
            "a-down",
            "insert",
            "s-insert",
            "c-insert",
            "delete",
            "s-delete",
            "c-delete",
            "c-a",
            "a-b",
            "a-f",
            "a-j",
            "c-h",
            "c-k",
            "c-l",
            "c-x",
            "c-c",
            "c-v",
            "c-y",
            "c-z",
            "c-s-z",
            "a-z",
        ] {
            assert!(parse_key_name(name).is_some(), "editor key must be bindable: {name}");
        }
    }

    #[test]
    fn profile_lines_parse() {
        let text =
            "# header\n\nsave\ndefine x = \\\n  # mid comment\n[undo] [redo]\n  # note\nquit";
        assert_eq!(
            profile_command_lines(text),
            vec![
                (3, "save".to_string()),
                (4, "define x = [undo] [redo]".to_string()),
                (8, "quit".to_string()),
            ]
        );

        assert_eq!(profile_command_lines("[delete] \\"), vec![(1, "[delete] \\".to_string())]);
    }

    #[test]
    fn profile_commands_parse_before_execution() {
        let Err(err) =
            parse_profile_commands("bind c-t = [undo]\nbind nope = [undo]\nbad", true, true)
        else {
            panic!("bad profile must fail preflight parse");
        };

        assert_eq!(err, (2, 2));
    }

    #[test]
    fn default_profile_resolves_and_binds_essentials() {
        let profile = include_str!("../../../../../../assets/ze2.pro");
        assert_eq!(profile, DEFAULT_PROFILE);
        let mut bindings = std::collections::HashMap::new();
        for (line_no, command) in profile_command_lines(profile) {
            let define_or_bind = command
                .strip_prefix("bind ")
                .map(|rest| (true, rest))
                .or_else(|| command.strip_prefix("define ").map(|rest| (false, rest)));

            let Some((is_bind, rest)) = define_or_bind else {
                assert!(
                    command_sequence_from_text(&command, true, true).is_some()
                        || command_from_text_with_modes(&command, true, true).is_some(),
                    "line {line_no}: did not parse: {command}"
                );
                continue;
            };

            let (name, body) = macro_name_and_body(rest)
                .unwrap_or_else(|| panic!("line {line_no}: bad name/body: {command}"));
            if is_bind {
                let key = parse_key_name(name)
                    .unwrap_or_else(|| panic!("line {line_no}: unknown key: {name}"));
                if !body.is_empty() {
                    bindings.insert(key, body.to_string());
                }
            }
            assert!(
                body.is_empty() || command_sequence_from_text(body, true, true).is_some(),
                "line {line_no}: body does not resolve: {body}"
            );
        }

        for (name, expected) in [
            ("a-b", Command::MarkBlock),
            ("a-l", Command::MarkLine),
            ("a-m", Command::MoveMark),
            ("a-c", Command::CopyMark),
            ("c-s", Command::Save),
            ("c-q", Command::Exit),
            ("c-w", Command::CloseFile),
            ("c-f", Command::Find),
        ] {
            let key = parse_key_name(name).unwrap();
            let body =
                bindings.get(&key).unwrap_or_else(|| panic!("default profile must bind {name}"));
            let seq = command_sequence_from_text(body, true, true).unwrap();
            assert_eq!(seq.len(), 1, "default profile bind {name} must be one command");
            assert!(seq[0].command == expected, "default profile bind {name}");
        }

        // Keys the pre-profile build handled itself must stay unbound so the
        // default profile does not change their behavior: home/end (text area's
        // smart cursor), tab/esc (indent, clear-selection), and a-f/a-u (the File
        // and Utils menu accelerators).
        for name in ["home", "end", "s-home", "s-end", "tab", "s-tab", "esc", "a-f", "a-u"] {
            let key = parse_key_name(name).unwrap();
            assert!(
                !bindings.contains_key(&key),
                "default profile must leave {name} to the text area or menubar"
            );
        }
    }

    #[test]
    fn default_profile_bindings_can_override_editor_and_menu_keys() {
        let mut tui = Tui::new().unwrap();
        let mut ctx = tui.create_context(None);
        let mut state = State::new().unwrap();
        state.loading_default_profile = true;

        run_profile_text(&mut ctx, &mut state, "bind a-f = [fill-mark]\nbind backspace = [noop]");

        let fill_key = kbmod::ALT | vk::F;
        assert!(state.user_bindings[&fill_key][0].command == Command::FillMark);
        assert!(state.key_bindings[&fill_key][0].command == Command::FillMark);
        assert!(!state.user_bindings.contains_key(&vk::BACK));
        assert!(state.key_bindings[&vk::BACK][0].command == Command::Noop);
    }

    #[test]
    fn default_profile_examples_are_valid() {
        for (line_no, line) in DEFAULT_PROFILE.lines().enumerate() {
            let example = line.trim_start().strip_prefix('#').map(str::trim_start);
            let Some(command) = example else {
                continue;
            };
            if !command.starts_with("bind ")
                && !command.starts_with("define ")
                && !command.starts_with("source ")
            {
                continue;
            }
            if command.contains('<') || command.contains("...") {
                continue;
            }

            let define_or_bind = command
                .strip_prefix("bind ")
                .map(|rest| (true, rest))
                .or_else(|| command.strip_prefix("define ").map(|rest| (false, rest)));

            let Some((is_bind, rest)) = define_or_bind else {
                assert!(
                    command_from_text_with_modes(command, true, true).is_some(),
                    "line {}: example does not parse: {command}",
                    line_no + 1
                );
                continue;
            };

            let (name, body) = macro_name_and_body(rest).unwrap_or_else(|| {
                panic!("line {}: bad example name/body: {command}", line_no + 1)
            });
            if is_bind {
                parse_key_name(name)
                    .unwrap_or_else(|| panic!("line {}: bad example key: {name}", line_no + 1));
            }
            assert!(
                body.is_empty() || command_sequence_from_text(body, true, true).is_some(),
                "line {}: example body does not resolve: {body}",
                line_no + 1
            );
        }
    }
}
