// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::framebuffer::IndexedColor;
use ze2::helpers::*;
use ze2::input::vk;
use ze2::tui::*;

use crate::commands::{
    Command, CommandArgs, CommandInvocation, autocomplete_command_suggestions_with_modes,
    command_from_text_with_modes, command_sequence_from_text, execute_command_sequence,
};
use crate::localization::{LocId, loc};
use crate::state::*;

pub fn draw_commandbar(ctx: &mut Context, state: &mut State) {
    let mut should_submit_input = false;
    let mut has_error = !state.command_bar_error.is_empty();

    ctx.table_begin("commandbar");
    ctx.attr_focus_well();
    ctx.attr_background_rgba(ctx.indexed(IndexedColor::Green));
    ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::BrightWhite));
    ctx.table_set_cell_gap(Size { width: 1, height: 0 });
    ctx.attr_intrinsic_size(Size { width: COORD_TYPE_SAFE_MAX, height: 1 });
    ctx.attr_padding(Rect::two(0, 1));
    {
        if has_error && ctx.keyboard_input().is_some() {
            state.command_bar_error.clear();
            has_error = false;
        }

        if ctx.contains_focus()
            && ctx.keyboard_input() == Some(vk::ESCAPE)
            && !state.wants_dialog()
            && !ctx.clipboard_ref().wants_host_sync()
        {
            state.command_bar_active = false;
            state.command_bar_error.clear();
            state.wants_editor_focus = true;
            ctx.needs_rerender();
            ctx.set_input_consumed();
        }

        ctx.table_next_row();
        let prompt = if ctx.contains_focus() { "▶" } else { "▷" };
        ctx.label("prompt", prompt);

        if ctx.editline("input", &mut state.command_bar_input) {
            state.command_bar_error.clear();
            has_error = false;
        }
        ctx.attr_background_rgba(ctx.indexed(IndexedColor::Green));
        ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::BrightWhite));
        ctx.attr_intrinsic_size(Size { width: COORD_TYPE_SAFE_MAX, height: 1 });
        ctx.inherit_focus();

        if ctx.contains_focus() {
            let autocomplete = commandbar_autocomplete_context(&state.command_bar_input);
            let suggestions = if let Some(prefix) = autocomplete.prefix {
                autocomplete_command_suggestions_with_modes(
                    prefix,
                    state.command_bar_include_vim_commands,
                    state.command_bar_include_emacs_commands,
                )
            } else {
                Vec::new()
            };

            if suggestions.is_empty() {
                state.command_bar_autocomplete_index = None;
                if ctx.is_focused() && ctx.consume_shortcut(vk::UP) {
                    state.command_bar_active = false;
                    state.command_bar_error.clear();
                    state.wants_editor_focus = true;
                    ctx.needs_rerender();
                }
            } else {
                let bg = ctx.indexed_alpha(IndexedColor::Background, 3, 4);
                let fg = ctx.contrasted(bg);

                let mut apply_autocomplete = false;
                // Handle keyboard navigation manually before the list takes it
                if ctx.is_focused() {
                    if ctx.consume_shortcut(vk::DOWN) {
                        let idx = state.command_bar_autocomplete_index.unwrap_or(0);
                        state.command_bar_autocomplete_index =
                            Some((idx + 1).min(suggestions.len() - 1));
                    } else if ctx.consume_shortcut(vk::UP) {
                        let idx = state.command_bar_autocomplete_index.unwrap_or(0);
                        state.command_bar_autocomplete_index = Some(idx.saturating_sub(1));
                    } else if ctx.consume_shortcut(vk::TAB) {
                        apply_autocomplete = true;
                    } else if ctx.consume_shortcut(vk::RETURN) {
                        should_submit_input = true;
                    }
                }

                if apply_autocomplete {
                    if let Some(idx) = state.command_bar_autocomplete_index {
                        if let Some(suggestion) = suggestions.get(idx) {
                            state.command_bar_input = autocomplete.apply(&suggestion.name);
                            state.command_bar_autocomplete_index = None;
                        }
                    } else if let Some(suggestion) = suggestions.first() {
                        state.command_bar_input = autocomplete.apply(&suggestion.name);
                        state.command_bar_autocomplete_index = None;
                    }
                }

                ctx.block_begin("suggestions");
                ctx.attr_float(FloatSpec {
                    anchor: Anchor::Last,
                    gravity_x: 0.0,
                    gravity_y: 1.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                });
                ctx.attr_border();
                ctx.attr_background_rgba(bg);
                ctx.attr_foreground_rgba(fg);
                {
                    for (idx, suggestion) in suggestions.iter().enumerate() {
                        let is_selected = state.command_bar_autocomplete_index == Some(idx);

                        ctx.next_block_id_mixin(idx as u64);
                        ctx.styled_label_begin("suggestion");
                        if is_selected {
                            ctx.attr_background_rgba(ctx.indexed(IndexedColor::White));
                            ctx.attr_foreground_rgba(ctx.indexed(IndexedColor::Black));
                        }
                        ctx.styled_label_add_text("  ");
                        let text = suggestion.display_text();
                        ctx.styled_label_add_text(&text);
                        ctx.styled_label_end();
                    }
                }
                ctx.block_end();

                // If user typed anything else, reset the autocomplete selection
                if ctx.keyboard_input().is_some()
                    && ctx.keyboard_input() != Some(vk::RETURN)
                    && ctx.keyboard_input() != Some(vk::TAB)
                {
                    state.command_bar_autocomplete_index = None;
                }
            }
        } else {
            state.command_bar_autocomplete_index = None;
        }

        if state.command_bar_focus {
            state.command_bar_focus = false;
            state.command_bar_active = true;
            ctx.steal_focus();
        }

        if ctx.contains_focus() {
            state.command_bar_active = true;

            if ctx.consume_shortcut(vk::RETURN) {
                should_submit_input = true;
            }
        }

        if has_error {
            ctx.block_begin("error_box");
            ctx.attr_float(FloatSpec {
                anchor: Anchor::Parent,
                gravity_x: 0.0,
                gravity_y: 1.0,
                offset_x: 0.0,
                offset_y: 0.0,
            });
            ctx.attr_border();
            let (bg, fg) = if state.command_bar_error_is_warning {
                (IndexedColor::Yellow, IndexedColor::Black)
            } else {
                (IndexedColor::Red, IndexedColor::BrightWhite)
            };
            ctx.attr_background_rgba(ctx.indexed(bg));
            ctx.attr_foreground_rgba(ctx.indexed(fg));
            ctx.attr_padding(Rect::two(0, 1));
            {
                ctx.label("error", &state.command_bar_error);
                ctx.attr_overflow(Overflow::TruncateTail);
            }
            ctx.block_end();
        }
    }
    ctx.table_end();

    if should_submit_input {
        submit_commandbar_input(ctx, state);
    }
}

struct CommandbarAutocompleteContext<'a> {
    prefix: Option<&'a str>,
    head: &'a str,
    tail: &'a str,
}

impl<'a> CommandbarAutocompleteContext<'a> {
    fn apply(&self, suggestion: &str) -> String {
        let mut text = String::with_capacity(self.head.len() + suggestion.len() + self.tail.len());
        text.push_str(self.head);
        text.push_str(suggestion);
        text.push_str(self.tail);
        text
    }
}

fn commandbar_autocomplete_context(input: &str) -> CommandbarAutocompleteContext<'_> {
    let trimmed = input.trim_start();
    if let Some(rest) = trimmed.strip_prefix("help") {
        let rest = rest.trim_start();
        if rest.is_empty() {
            return CommandbarAutocompleteContext { prefix: Some(""), head: input, tail: "" };
        }

        let arg_len = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let arg = &rest[..arg_len];
        let head_len = input.len() - rest.len();
        let tail = &rest[arg_len..];
        return CommandbarAutocompleteContext { prefix: Some(arg), head: &input[..head_len], tail };
    }

    if let Some(head_len) = input.find(char::is_whitespace) {
        let head = &input[..head_len];
        let tail = &input[head_len..];
        return CommandbarAutocompleteContext { prefix: Some(head), head: "", tail };
    }

    if !input.is_empty() {
        return CommandbarAutocompleteContext { prefix: Some(input), head: "", tail: "" };
    }

    CommandbarAutocompleteContext { prefix: None, head: "", tail: "" }
}

fn submit_commandbar_input(ctx: &mut Context, state: &mut State) {
    // Own the input up front: every branch below mutates "state", which a borrow
    // of "command_bar_input" would forbid.
    let input = state.command_bar_input.trim().to_string();
    let include_vim = state.command_bar_include_vim_commands;
    let include_emacs = state.command_bar_include_emacs_commands;

    let invocations =
        if let Some(sequence) = command_sequence_from_text(&input, include_vim, include_emacs) {
            Some(sequence)
        } else if let Some(invocation) =
            command_from_text_with_modes(&input, include_vim, include_emacs)
        {
            Some(vec![invocation])
        } else if state.macros.contains_key(&input) {
            // A bare name that is not a built-in but is a saved macro runs that macro.
            // Built-ins win any collision because they are matched first.
            Some(vec![CommandInvocation {
                command: Command::RunMacro,
                args: CommandArgs { argument: Some(input.clone()), ..Default::default() },
            }])
        } else {
            None
        };

    let Some(invocations) = invocations else {
        if !input.is_empty() {
            state.command_bar_input.clear();
            state.command_bar_error = loc(LocId::CommandBarUnknownCommand).to_string();
            state.command_bar_error_is_warning = false;
            ctx.needs_rerender();
        }
        return;
    };

    state.command_bar_input.clear();
    state.command_bar_error.clear();
    state.command_bar_active = false;
    state.wants_editor_focus = true;
    // Fresh top-level run: clear any abort left by a previous macro.
    state.macro_abort = false;
    execute_command_sequence(ctx, state, invocations);
}

#[cfg(test)]
mod tests {
    use super::commandbar_autocomplete_context;

    #[test]
    fn autocomplete_uses_first_command_token() {
        let ctx = commandbar_autocomplete_context("set-editor-color ");
        assert_eq!(ctx.prefix, Some("set-editor-color"));
        assert_eq!(ctx.apply("set-editor-color"), "set-editor-color ");
    }

    #[test]
    fn autocomplete_keeps_help_argument_tail() {
        let ctx = commandbar_autocomplete_context("help about ");
        assert_eq!(ctx.prefix, Some("about"));
        assert_eq!(ctx.apply("about"), "help about ");
    }

    #[test]
    fn autocomplete_offers_help_with_empty_argument() {
        let ctx = commandbar_autocomplete_context("help ");
        assert_eq!(ctx.prefix, Some(""));
        assert_eq!(ctx.apply("save"), "help save");
    }

    #[test]
    fn autocomplete_ignores_multi_word_non_help_input() {
        let ctx = commandbar_autocomplete_context("set editor");
        assert_eq!(ctx.prefix, Some("set"));
        assert_eq!(ctx.apply("set-editor-color"), "set-editor-color editor");
    }

    #[test]
    fn autocomplete_help_uses_second_token_prefix() {
        let ctx = commandbar_autocomplete_context("help ab");
        assert_eq!(ctx.prefix, Some("ab"));
        assert_eq!(ctx.apply("about"), "help about");
    }
}
