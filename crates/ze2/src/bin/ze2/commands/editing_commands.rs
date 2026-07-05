// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ze2::buffer::{CursorMovement, MoveLineDirection, TextBuffer, TextMarkKind};
use ze2::helpers::{CoordType, Point};
use ze2::tui::Context;

use super::{Command, CommandArgs, CommandDefinition, CommandFocusTarget};
use crate::localization::{LocId, loc};
use crate::state::State;

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        command: Command::Undo,
        names: &["undo"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::EditUndo),
        default_focus_target: CommandFocusTarget::Default,
        handler: undo,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Redo,
        names: &["redo"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: Some(LocId::EditRedo),
        default_focus_target: CommandFocusTarget::Default,
        handler: redo,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Cut,
        names: &["cut"],
        namesVim: &["delete"],
        namesEmacs: &["kill-region"],
        loc_id: Some(LocId::EditCut),
        default_focus_target: CommandFocusTarget::Default,
        handler: cut,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Copy,
        names: &["copy"],
        namesVim: &["yank"],
        namesEmacs: &["kill-ring-save"],
        loc_id: Some(LocId::EditCopy),
        default_focus_target: CommandFocusTarget::Default,
        handler: copy,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Paste,
        names: &["paste"],
        namesVim: &["put"],
        namesEmacs: &["clipboard-yank"],
        loc_id: Some(LocId::EditPaste),
        default_focus_target: CommandFocusTarget::Default,
        handler: paste,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectAll,
        names: &["select-all"],
        namesVim: &[],
        namesEmacs: &["mark-whole-buffer"],
        loc_id: Some(LocId::EditSelectAll),
        default_focus_target: CommandFocusTarget::Default,
        handler: select_all,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectLine,
        names: &["select-line", "line"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::InsertText,
        names: &["insert"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: insert_text,
        argument_hint: Some("<text>"),
    },
    CommandDefinition {
        command: Command::InsertLine,
        names: &["insert-line", "il"],
        namesVim: &[],
        namesEmacs: &["open-line"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: insert_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SplitLine,
        names: &["split-line", "split", "sp"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: split_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::JoinLine,
        names: &["join-line", "join", "jo"],
        namesVim: &["join"],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: join_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::FirstNonblank,
        names: &["first-nonblank", "fn"],
        namesVim: &[],
        namesEmacs: &["back-to-indentation"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: first_nonblank,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveToWordBegin,
        names: &["begin-word", "wb"],
        namesVim: &[],
        namesEmacs: &["backward-word"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: begin_word,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveToWordEnd,
        names: &["end-word", "we"],
        namesVim: &[],
        namesEmacs: &["forward-word"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: end_word,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveToVisualLineBegin,
        names: &["begin-visual-line"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: go_to_visual_line_begin,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveToVisualLineEnd,
        names: &["end-visual-line"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: go_to_visual_line_end,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::TabWord,
        names: &["tab-word", "tw"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: tab_word,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::BacktabWord,
        names: &["backtab-word", "bw"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: backtab_word,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MarkLine,
        names: &["mark-line", "ml"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: mark_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MarkChar,
        names: &["mark-char", "mc"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: mark_char,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MarkBlock,
        names: &["mark-block", "mb"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: mark_block,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MarkSelection,
        names: &["mark-selection", "ms"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: mark_selection,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::Unmark,
        names: &["unmark", "um"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: unmark,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::CopyMark,
        names: &["copy-mark", "cm"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: copy_mark,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveMark,
        names: &["move-mark", "mm"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_mark,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::DeleteMark,
        names: &["delete-mark", "dm"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: delete_mark,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::FillMark,
        names: &["fill-mark", "fm"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: fill_mark,
        argument_hint: Some("<char>"),
    },
    CommandDefinition {
        command: Command::OverlayBlock,
        names: &["overlay-block", "ob"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: overlay_block,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::ShiftLeft,
        names: &["shift-left", "sl"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: shift_left,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::ShiftRight,
        names: &["shift-right", "sr"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: shift_right,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::CopyToCmd,
        names: &["copy-to-command", "ct"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: copy_to_command,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::CopyFromCmd,
        names: &["copy-from-command", "cf"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: copy_from_command,
        argument_hint: None,
    },
    // Cursor movement, selection, and deletion primitives, so a profile can bind
    // any key to an editor action. Each is a thin wrapper over a buffer method.
    CommandDefinition {
        command: Command::MoveLeft,
        names: &["move-left"],
        namesVim: &[],
        namesEmacs: &["backward-char"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_left,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveRight,
        names: &["move-right"],
        namesVim: &[],
        namesEmacs: &["forward-char"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_right,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveToDocumentBegin,
        names: &["move-document-begin", "document-begin"],
        namesVim: &[],
        namesEmacs: &["beginning-of-buffer"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_document_begin,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveToDocumentEnd,
        names: &["move-document-end", "document-end"],
        namesVim: &[],
        namesEmacs: &["end-of-buffer"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_document_end,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveLinesUp,
        names: &["move-lines-up"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_lines_up,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveLinesDown,
        names: &["move-lines-down"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_lines_down,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectLeft,
        names: &["select-left"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_left,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectRight,
        names: &["select-right"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_right,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectWordLeft,
        names: &["select-word-left"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_word_left,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectWordRight,
        names: &["select-word-right"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_word_right,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectToDocumentBegin,
        names: &["select-document-begin"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_document_begin,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectToDocumentEnd,
        names: &["select-document-end"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_document_end,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::DeleteForward,
        names: &["delete-forward"],
        namesVim: &[],
        namesEmacs: &["delete-char"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: delete_forward,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::DeleteBackward,
        names: &["delete-backward"],
        namesVim: &[],
        namesEmacs: &["delete-backward-char"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: delete_backward,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::DeleteWordForward,
        names: &["delete-word-forward"],
        namesVim: &[],
        namesEmacs: &["kill-word"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: delete_word_forward,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::DeleteWordBackward,
        names: &["delete-word-backward"],
        namesVim: &[],
        namesEmacs: &["backward-kill-word"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: delete_word_backward,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::DeleteLine,
        names: &["delete-line"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: delete_line,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::DeleteToLineEnd,
        names: &["delete-to-line-end"],
        namesVim: &[],
        namesEmacs: &["kill-line"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: delete_to_line_end,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::ToggleOvertype,
        names: &["toggle-overtype", "overtype"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: toggle_overtype,
        argument_hint: None,
    },
    // Vertical movement and paging. These keep the sticky column in State, so
    // they read and write State directly instead of only the buffer.
    CommandDefinition {
        command: Command::MoveUp,
        names: &["move-up"],
        namesVim: &[],
        namesEmacs: &["previous-line"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_up,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::MoveDown,
        names: &["move-down"],
        namesVim: &[],
        namesEmacs: &["next-line"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: move_down,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::PageUp,
        names: &["page-up"],
        namesVim: &[],
        namesEmacs: &["scroll-down"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: page_up,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::PageDown,
        names: &["page-down"],
        namesVim: &[],
        namesEmacs: &["scroll-up"],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: page_down,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectUp,
        names: &["select-up"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_up,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectDown,
        names: &["select-down"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_down,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectPageUp,
        names: &["select-page-up"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_page_up,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectPageDown,
        names: &["select-page-down"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_page_down,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectToVisualLineBegin,
        names: &["select-line-begin"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_line_begin,
        argument_hint: None,
    },
    CommandDefinition {
        command: Command::SelectToVisualLineEnd,
        names: &["select-line-end"],
        namesVim: &[],
        namesEmacs: &[],
        loc_id: None,
        default_focus_target: CommandFocusTarget::Default,
        handler: select_line_end,
        argument_hint: None,
    },
];

fn undo(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().undo();
    }
}

fn redo(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().redo();
    }
}

fn cut(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cut(ctx.clipboard_mut());
    }
}

fn copy(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().copy(ctx.clipboard_mut());
    }
}

fn paste(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().paste(ctx.clipboard_ref(), false);
    }
}

fn select_all(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().select_all();
    }
}

fn select_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().select_line();
    }
}

fn insert_text(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(text) = args.argument
        && let Some(doc) = state.documents.active()
    {
        doc.buffer.borrow_mut().write_canon_smart(text.as_bytes());
    }
}

fn insert_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().insert_line_below();
    }
}

fn split_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().split_line();
    }
}

fn join_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().join_next_line();
    }
}

fn first_nonblank(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_to_first_nonblank();
    }
}

fn begin_word(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_to_begin_word();
    }
}

fn end_word(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_to_end_word();
    }
}

fn go_to_visual_line_begin(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let y = doc.buffer.borrow().cursor_visual_pos().y;
        doc.buffer.borrow_mut().cursor_move_to_visual(Point { x: 0, y });
    }
}

fn go_to_visual_line_end(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let y = doc.buffer.borrow().cursor_visual_pos().y;
        doc.buffer.borrow_mut().cursor_move_to_visual(Point { x: CoordType::MAX, y });
    }
}

fn tab_word(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_delta(CursorMovement::Word, 1);
    }
}

fn backtab_word(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_delta(CursorMovement::Word, -1);
    }
}

fn move_left(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        // With a selection, a plain move collapses to its near edge, matching the
        // text area; otherwise it steps one grapheme.
        if let Some((beg, _)) = buf.selection_range() {
            buf.cursor_move_to_logical(beg.logical_pos);
        } else {
            buf.cursor_move_delta(CursorMovement::Grapheme, -1);
        }
    }
}

fn move_right(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        if let Some((_, end)) = buf.selection_range() {
            buf.cursor_move_to_logical(end.logical_pos);
        } else {
            buf.cursor_move_delta(CursorMovement::Grapheme, 1);
        }
    }
}

fn move_document_begin(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_to_logical(Point::default());
    }
}

fn move_document_end(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().cursor_move_to_logical(Point::MAX);
    }
}

fn move_lines_up(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().move_selected_lines(MoveLineDirection::Up);
    }
}

fn move_lines_down(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().move_selected_lines(MoveLineDirection::Down);
    }
}

fn select_left(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().selection_update_delta(CursorMovement::Grapheme, -1);
    }
}

fn select_right(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().selection_update_delta(CursorMovement::Grapheme, 1);
    }
}

fn select_word_left(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().selection_update_delta(CursorMovement::Word, -1);
    }
}

fn select_word_right(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().selection_update_delta(CursorMovement::Word, 1);
    }
}

fn select_document_begin(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().selection_update_logical(Point::default());
    }
}

fn select_document_end(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().selection_update_logical(Point::MAX);
    }
}

fn delete_forward(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().delete(CursorMovement::Grapheme, 1);
    }
}

fn delete_backward(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().delete(CursorMovement::Grapheme, -1);
    }
}

fn delete_word_forward(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().delete(CursorMovement::Word, 1);
    }
}

fn delete_word_backward(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().delete(CursorMovement::Word, -1);
    }
}

fn delete_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().delete_line();
    }
}

fn delete_to_line_end(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().delete_to_end_of_line(ctx.clipboard_mut());
    }
}

fn toggle_overtype(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        let overtype = buf.is_overtype();
        buf.set_overtype(!overtype);
    }
}

// Recapture the sticky column unless this continues a run of vertical moves,
// i.e. the cursor is still exactly where the previous vertical move left it.
// Typing, a click, or a horizontal move all shift the cursor, which resets it.
fn vertical_move_begin(state: &mut State) -> Option<()> {
    let cursor = state.documents.active()?.buffer.borrow().cursor_visual_pos();
    if state.last_vertical_cursor != Some(cursor) {
        state.preferred_column = cursor.x;
    }
    Some(())
}

// Record where a vertical move left the cursor, so the next one can tell whether
// it continues the run (see vertical_move_begin).
fn vertical_move_end(state: &mut State) {
    state.last_vertical_cursor =
        state.documents.active().map(|doc| doc.buffer.borrow().cursor_visual_pos());
}

// The visual row a vertical move starts from. With a selection active it is the
// near edge -- top for an upward move, bottom for a downward one -- and the
// second field is that edge's column, so the caller can adopt it as the sticky
// column (paging and stepping then collapse a selection to the same corner).
// With no selection it is the cursor row and None (keep the existing sticky
// column).
fn vertical_move_base_row(buf: &TextBuffer, downward: bool) -> (CoordType, Option<CoordType>) {
    match buf.selection_range() {
        Some((beg, end)) => {
            let edge = if downward { end } else { beg };
            (edge.visual_pos.y, Some(edge.visual_pos.x))
        }
        None => (buf.cursor_visual_pos().y, None),
    }
}

fn move_up(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let Some(()) = vertical_move_begin(state) else { return };
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        let (base_y, edge_x) = vertical_move_base_row(&buf, false);
        if let Some(x) = edge_x {
            state.preferred_column = x;
        }
        let mut x = state.preferred_column;
        let y = base_y - 1;
        if y < 0 {
            x = 0;
            state.preferred_column = 0;
        }
        buf.cursor_move_to_visual(Point { x, y });
    }
    vertical_move_end(state);
}

fn move_down(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let Some(()) = vertical_move_begin(state) else { return };
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        let (base_y, edge_x) = vertical_move_base_row(&buf, true);
        if let Some(x) = edge_x {
            state.preferred_column = x;
        }
        let mut x = state.preferred_column;
        let y = base_y + 1;
        if y >= buf.visual_line_count() {
            x = CoordType::MAX;
        }
        buf.cursor_move_to_visual(Point { x, y });
        if x == CoordType::MAX {
            state.preferred_column = buf.cursor_visual_pos().x;
        }
    }
    vertical_move_end(state);
}

fn page_up(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let Some(()) = vertical_move_begin(state) else { return };
    let height = (state.editor_viewport_height - 1).max(1);
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        // Page from the caret, matching the text area's PgUp. Unlike Up, it does
        // not collapse to a selection edge; it only resets the sticky column when
        // the caret is already on the first line.
        if buf.cursor_visual_pos().y == 0 {
            state.preferred_column = 0;
        }
        let y = buf.cursor_visual_pos().y - height;
        buf.cursor_move_to_visual(Point { x: state.preferred_column, y });
    }
    vertical_move_end(state);
}

fn page_down(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let Some(()) = vertical_move_begin(state) else { return };
    let height = (state.editor_viewport_height - 1).max(1);
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        // Page from the caret, matching the text area's PgDn: reset the sticky
        // column to the line end only when already on the last line.
        if buf.cursor_visual_pos().y >= buf.visual_line_count() - 1 {
            state.preferred_column = CoordType::MAX;
        }
        let y = buf.cursor_visual_pos().y + height;
        buf.cursor_move_to_visual(Point { x: state.preferred_column, y });
        if state.preferred_column == CoordType::MAX {
            state.preferred_column = buf.cursor_visual_pos().x;
        }
    }
    vertical_move_end(state);
}

fn select_up(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let Some(()) = vertical_move_begin(state) else { return };
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        if buf.cursor_visual_pos().y == 0 {
            state.preferred_column = 0;
        }
        let y = buf.cursor_visual_pos().y - 1;
        buf.selection_update_visual(Point { x: state.preferred_column, y });
    }
    vertical_move_end(state);
}

fn select_down(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let Some(()) = vertical_move_begin(state) else { return };
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        if buf.cursor_visual_pos().y >= buf.visual_line_count() - 1 {
            state.preferred_column = CoordType::MAX;
        }
        let y = buf.cursor_visual_pos().y + 1;
        buf.selection_update_visual(Point { x: state.preferred_column, y });
        if state.preferred_column == CoordType::MAX {
            state.preferred_column = buf.cursor_visual_pos().x;
        }
    }
    vertical_move_end(state);
}

fn select_page_up(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let Some(()) = vertical_move_begin(state) else { return };
    let height = (state.editor_viewport_height - 1).max(1);
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        if buf.cursor_visual_pos().y == 0 {
            state.preferred_column = 0;
        }
        let y = buf.cursor_visual_pos().y - height;
        buf.selection_update_visual(Point { x: state.preferred_column, y });
    }
    vertical_move_end(state);
}

fn select_page_down(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    let Some(()) = vertical_move_begin(state) else { return };
    let height = (state.editor_viewport_height - 1).max(1);
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        if buf.cursor_visual_pos().y >= buf.visual_line_count() - 1 {
            state.preferred_column = CoordType::MAX;
        }
        let y = buf.cursor_visual_pos().y + height;
        buf.selection_update_visual(Point { x: state.preferred_column, y });
        if state.preferred_column == CoordType::MAX {
            state.preferred_column = buf.cursor_visual_pos().x;
        }
    }
    vertical_move_end(state);
}

fn select_line_begin(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        let y = buf.cursor_visual_pos().y;
        buf.selection_update_visual(Point { x: 0, y });
    }
}

fn select_line_end(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut buf = doc.buffer.borrow_mut();
        let y = buf.cursor_visual_pos().y;
        buf.selection_update_visual(Point { x: CoordType::MAX, y });
    }
}

fn mark_line(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().mark(TextMarkKind::Line);
    }
}

fn mark_char(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().mark(TextMarkKind::Char);
    }
}

fn mark_block(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let mut tb = doc.buffer.borrow_mut();
        if tb.is_word_wrap_enabled() {
            state.command_bar_error = loc(LocId::CommandMarkBlockWordWrapEnabled).to_string();
            state.command_bar_error_is_warning = false;
            return;
        }
        tb.mark(TextMarkKind::Block);
    }
}

fn mark_selection(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().mark_selection();
    }
}

fn unmark(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().clear_mark();
    }
}

fn copy_mark(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().copy_mark_to_cursor();
    }
}

fn move_mark(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().move_mark_to_cursor();
    }
}

fn delete_mark(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().delete_mark(ctx.clipboard_mut());
    }
}

fn fill_mark(_ctx: &mut Context, state: &mut State, args: CommandArgs) {
    if let Some(text) = args.argument {
        if let Some(doc) = state.documents.active() {
            doc.buffer.borrow_mut().fill_mark(text.as_bytes());
        }
        return;
    }

    // No fill character was given (for example a bare "[fill-mark]" binding), so
    // prompt for one, but only when there is a mark to fill.
    if state.documents.active().is_some_and(|doc| doc.buffer.borrow().has_mark()) {
        state.wants_fill_mark = true;
    }
}

fn overlay_block(ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().overlay_block_from_clipboard(ctx.clipboard_ref());
    }
}

fn shift_left(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().shift_block_mark(false);
    }
}

fn shift_right(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().shift_block_mark(true);
    }
}

fn copy_to_command(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        let line = doc.buffer.borrow().current_line_text();
        state.command_bar_input = String::from_utf8_lossy(&line).into_owned();
        state.command_bar_active = true;
        state.command_bar_focus = true;
        state.wants_editor_focus = false;
    }
}

fn copy_from_command(_ctx: &mut Context, state: &mut State, _args: CommandArgs) {
    if let Some(doc) = state.documents.active() {
        doc.buffer.borrow_mut().write_canon(state.command_bar_input.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use ze2::buffer::TextBuffer;
    use ze2::helpers::Point;

    use super::vertical_move_base_row;

    #[test]
    fn vertical_move_collapses_to_selection_edge() {
        let mut buf = TextBuffer::new(false).unwrap();
        buf.set_crlf(false);
        buf.set_insert_final_newline(false);
        buf.write_raw(b"line one\nline two\nline three");

        // No selection: start from the cursor row, leave the sticky column alone.
        buf.cursor_move_to_visual(Point { x: 4, y: 1 });
        assert_eq!(vertical_move_base_row(&buf, true), (1, None));
        assert_eq!(vertical_move_base_row(&buf, false), (1, None));

        // Selection spanning rows 0..2: a downward move starts at the bottom edge
        // and an upward move at the top edge, each adopting that edge's column.
        buf.cursor_move_to_visual(Point { x: 2, y: 0 });
        buf.selection_update_visual(Point { x: 5, y: 2 });
        assert_eq!(vertical_move_base_row(&buf, true), (2, Some(5)));
        assert_eq!(vertical_move_base_row(&buf, false), (0, Some(2)));
    }
}
