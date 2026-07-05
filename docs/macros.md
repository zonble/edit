# Macros and Profiles

ze2 carries the PE editor family's idea of a profile: a plain-text file of
commands that name reusable command sequences (macros) and bind them to keys.
The commands exposed in the command bar are the building blocks, so most editor
actions available as commands can be scripted.

This guide covers the profile file, the command-sequence syntax, key names, how
bindings are dispatched, and the macro/record features. For the meaning of the
mark commands used in many examples, see [mark-semantics.md](mark-semantics.md).

## Where profiles come from

There are two profiles, loaded in this order:

1. The built-in default, [ze2.pro](../assets/ze2.pro), compiled into the binary. It always
   loads first and defines the standard key bindings.
2. Your profile, the file named by the `ZE2_PROFILE` environment variable. It
   loads second, so it can override or remove anything the default set.

```sh
ZE2_PROFILE=~/.ze2.pro ze2 notes.txt      # load a profile for one run
export ZE2_PROFILE="$HOME/.ze2.pro"       # or set it for the shell
```

A profile is loaded only through `ZE2_PROFILE`. A path on the command line
(`ze2 notes.txt`) opens that file for editing; it is not read as a profile, even
if it ends in `.pro`. You can also pull in another profile from inside one with
`source <path>`.

## File format

- One command per line, the same text you would type in the command bar.
- Lines beginning with `#` are comments.
- A line ending in `\` continues onto the next line.

Three directives are specific to profiles:

```text
define <name> = [cmd] [cmd] ...   name a reusable command sequence (a macro)
bind <key>   = [cmd] [cmd] ...    bind a key to a command sequence
source <path>                     load another profile file
```

An empty body removes what was there, PE-style:

```text
define dup-line =                 remove the macro named dup-line
bind c-w =                        remove the binding on Ctrl-W entirely
```

## Command sequences

A sequence is one or more bracketed steps run left to right:

```text
[mark-line] [copy-mark] [unmark]
```

Each step is a command, optionally with an argument or a repeat count.

- Argument: the first word in the brackets is the command, the rest is its
  argument. `[find foo]`, `[goto 42]`, `[fill-mark -]`. The argument may be
  punctuation or a full-width character: `[fill-mark *]`, `[fill-mark 黃]`.
- Repeat count: a leading integer runs the step N times. `[3 undo]` undoes three
  times; `[5 insert-line]` opens five blank lines.

Limits of the v1 syntax: a bracketed argument cannot contain `]`, and there is
no escaping. If any step in a sequence fails to parse, the whole sequence is
rejected before anything runs, so a macro never half-executes.

## Key names

A key name is optional modifier prefixes followed by a base key, case
insensitive:

- Modifiers: `c-` (Ctrl), `a-` (Alt), `s-` (Shift). Combine them: `c-s-s`,
  `c-a-up`.
- Base: a letter or digit (`a`, `7`); a named key (`enter`, `esc`, `tab`,
  `space`, `up`, `down`, `left`, `right`, `home`, `end`, `pgup`, `pgdn`, `ins`,
  `del`, `backspace`); or a function key `f1`-`f24`.
- Punctuation that would clash with the grammar has an alias: `comma`, `period`
  (or `dot`), `colon`, `semicolon`, `slash`, `question`, `exclamation` (`bang`),
  `apostrophe` (`quote`), `lbracket`, `rbracket`, `lbrace`, `rbrace`, `lt`
  (`less`), `gt` (`greater`).

Because the base is case insensitive, write Shift explicitly: `s-a`, not `A`.

## How bindings are dispatched

Every `bind`, whether in the default or your profile, takes effect before the
menubar and the text area see the key. That is what lets a profile override any
key, including editor editing keys (arrows, `Ctrl-C`) and menu accelerators
(`Alt-F`). When two bindings target the same key, the one loaded later wins, so
your `ZE2_PROFILE` overrides the default.

Three special cases:

- `bind <key> = [noop]` reserves a key for the editor. The default profile uses
  it to document editor-owned keys (backspace, arrows, `Ctrl-C`, ...) without
  taking them over. Use it in your own profile to disable a key.
- `bind <key> =` (empty) removes a binding entirely, including a default one.
- Binding a key the text area uses (for example `Ctrl-V` or an arrow) shadows
  that editing action. Unless you mean to replace it, pick a free key such as
  `Ctrl-D/T/K/B`, `Ctrl-Space`, or `F5`-`F8` / `F10`-`F12`.

### macOS Option keys

`a-` maps to the Option key on macOS, but Option composes special characters by
default, so `a-` bindings only reach ze2 when the terminal is set to send Option
as Meta:

- Terminal.app: "Use Option as Meta Key"
- iTerm2: set the Option key to "Esc+"
- Ghostty: `macos-option-as-alt = true`
- kitty: `macos_option_as_alt yes`

Without that, prefer `c-` bindings, which arrive unchanged. When you bind an
`a-` key interactively in the command bar, ze2 reminds you of this once.

## Defining and running macros

Name a sequence with `define`, then run it by name with `[macro <name>]`, either
from a binding or from another sequence:

```text
define dup-line = [mark-line] [copy-mark] [unmark]
bind c-d = [macro dup-line]
```

A macro may call another macro, so recursion is possible; ze2 caps the nesting
depth (32) so a self-referential macro stops instead of looping forever.

`copy-mark` inserts the marked text at the cursor (it does not use the
clipboard), so duplicating a line needs no `paste`. The trailing `unmark`
matters: marking the same kind again extends the existing mark, so without it a
repeated `Ctrl-D` grows the mark and the block doubles each time.

## Running a region as a macro

`execute` (aliased `run-region`) runs the current selection, or the current line
if nothing is selected, as a command sequence. It turns the buffer itself into a
macro scratchpad: type `[reflow] [save]` on a line, then run `execute` on it.

```text
bind c-space = [execute]
```

## Record and replay

`record` toggles recording. While it is on, ze2 captures command-bar, menu,
shortcut, and binding actions (not ordinary typing or cursor motion). `replay`
(aliased `play-macro`) runs the captured steps back.

```text
bind f8  = [record]
bind f10 = [replay]
```

## Marks and fill-mark

Mark commands (`mark-block`, `mark-line`, `mark-char`, `move-mark`, `copy-mark`,
`fill-mark`, `unmark`) drive PE-style block operations; their exact coordinate
rules are in [mark-semantics.md](mark-semantics.md). Two behaviors surprise
people:

- A block mark has two corners, both set by pressing `mark-block`. It does not
  grow as you move the cursor. Press `mark-block` at one corner, move to the
  opposite corner, press `mark-block` again to extend it, then operate.
- `fill-mark` fills the marked region with a character it takes as an argument.
  `[fill-mark -]` fills with `-`. A bare `[fill-mark]` has no character, so it
  opens a small dialog to ask for one (when a mark exists). The character may be
  full-width; ze2 pads a leftover column with a space when a wide character does
  not divide the region evenly.

## Editor-action commands

So a profile can bind any key to an editor action, these primitives exist as
commands (each wraps the same buffer operation the text area uses):

```text
move-left / move-right              step one grapheme (collapses a selection)
move-document-begin / -end          jump to the start or end of the buffer
move-lines-up / move-lines-down     move the selected lines up or down
select-left / select-right          extend the selection one grapheme
select-word-left / -word-right      extend the selection one word
select-document-begin / -end        extend the selection to buffer start or end
delete-forward / delete-backward    delete one grapheme ahead of or behind the cursor
delete-word-forward / -word-backward delete one word ahead or behind
delete-line                         delete the current line
delete-to-line-end                  delete from the cursor to end of line
```

Vertical movement and paging are commands too: `move-up`, `move-down`,
`page-up`, `page-down`, and their selection variants `select-up`, `select-down`,
`select-page-up`, `select-page-down`, plus `select-line-begin` / `select-line-end`.
These keep the sticky column (the goal column vertical movement preserves) and
page by the editor's visible height, so a profile can rebind Up/Down/Page or put
them on other keys and get the same feel as the built-in behavior.

They join the commands already available (`move-to-word-begin`, `move-to-word-end`,
`begin-visual-line`, `end-visual-line`, `select-line`, `copy`, `cut`, `paste`,
`undo`, `redo`, `split-line`, `join-line`, `shift-left`, `shift-right`, ...). The
default profile binds most movement, selection, and deletion keys to these, so
they are fully redefinable. For example, emacs-style motion on free keys:

```text
bind c-b = [move-left]
bind c-f = [move-right]
bind c-p = [move-up]
bind a-d = [delete-word-forward]
```

To keep the same behavior as the pre-profile build, the default profile leaves a
few keys to the text area and menubar rather than binding them: Ctrl+Up/Down
(scroll) and Ctrl+Alt+Up/Down (add-cursor, unimplemented); Home/End and
Shift+Home/End (the text area's word-wrap- and indent-aware cursor); Tab/Shift+Tab
(indent) and Esc (clear selection); and Alt+F/E/G/V/U/H (the menu accelerators).
Bind any of them in your own profile to take them over -- for instance
`bind a-f = [fill-mark]` or `bind esc = [unmark]`.

## More examples

```text
# Uppercase or lowercase the current line in place.
define upper-line = [mark-line] [uppercase] [unmark]
define lower-line = [mark-line] [lowercase] [unmark]
bind c-t   = [macro upper-line]
bind c-s-t = [macro lower-line]

# Reflow the current paragraph, then save.
define tidy = [reflow] [save]
bind f5 = [macro tidy]

# Toggle word wrap and the ruler for prose, and show the word count.
bind f6 = [word-wrap] [toggle-ruler]
bind f7 = [word-count]

# Stamp a date on a fresh line below.
define stamp = [insert-line] [date]
bind c-k = [macro stamp]
```

The command bar autocompletes command names and shows each one's argument hint,
which is the authoritative list of what you can put in a sequence.

## Troubleshooting

- An `a-` binding does nothing on macOS: the terminal is not sending Option as
  Meta. See the macOS Option note above.
- `fill-mark` fills nothing: there is no mark (mark first), or a block mark has
  only one corner (extend it with a second `mark-block`).
- `Alt-F` opens the File menu instead of running your binding: an active status
  message in the command bar can block the before-editor bindings; press any key
  to dismiss it, or see that the binding loaded (a profile line that fails is
  reported and skipped).
