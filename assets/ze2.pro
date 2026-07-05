# Default ze2 profile: the built-in editor key bindings.
#
# ze2 compiles this file into the binary and runs it at startup, then loads the
# file named by the ZE2_PROFILE environment variable, which overrides anything
# here. Both run before the menubar and text area, so a bind can override any
# key, including editor and menu keys.
#
# Directives:
#   define <name> = [cmd] [cmd] ...   name a reusable command sequence
#   bind <key>   = [cmd] [cmd] ...    bind a key to a command sequence
#   source <path>                     load another profile file
# An empty body removes it ("bind c-w ="); "[noop]" reserves a key for the
# editor without stealing it (used below to document editor-owned keys).
#
# Key names: c- (Ctrl), a- (Alt), s- (Shift), combinable and case-insensitive,
# over a letter, digit, named key (enter, esc, tab, space, up, down, left,
# right, home, end, pgup, pgdn, ins, del), punctuation alias (comma, slash,
# lbracket, ...), or f1-f24.
#
# On macOS, a- is the Option key and reaches ze2 only when the terminal sends
# Option as Meta (Terminal.app "Use Option as Meta Key"; iTerm2 Option = "Esc+";
# Ghostty macos-option-as-alt = true; kitty macos_option_as_alt yes). Otherwise
# prefer c- bindings, which arrive unchanged.
#
# Full macro and profile guide, with examples: docs/macros.md.

# marks
# a-f (File menu) and a-u (Utils menu) are intentionally left to the menubar, as
# on the pre-profile build; bind them in your ZE2_PROFILE if you want fill-mark
# and unmark on those keys. On macOS, Option+Left arrives as a-b, so it marks a
# block instead of moving a word back (word nav stays on Ctrl+Left/Right).
bind a-b   = [mark-block]
bind a-l   = [mark-line]
bind a-m   = [move-mark]
bind a-c   = [copy-mark]

# file / search / app
bind c-n   = [new]
bind c-o   = [open]
bind c-s   = [save]
bind c-s-s = [save-as]
bind c-w   = [close]
bind c-p   = [go-to-file]
bind c-q   = [exit]
bind c-g   = [goto]
bind c-f   = [find]
bind c-r   = [replace]

# delete / edit
bind backspace = [delete-backward]
bind c-backspace = [delete-line]
bind c-h = [delete-word-backward]
bind delete = [delete-forward]
bind c-delete = [delete-word-forward]
bind s-delete = [cut]
bind c-k = [delete-to-line-end]

# newline
# Tab/Shift+Tab (indent) and Esc (clear selection) are left to the text area, as
# on the pre-profile build. Bind them in your ZE2_PROFILE to override.
bind enter = [split-line]
bind f9 = [split-line]

# navigation
bind c-home = [move-document-begin]
bind c-end = [move-document-end]
bind pgup = [page-up]
bind pgdn = [page-down]
bind left = [move-left]
bind right = [move-right]
bind c-left = [begin-word]
bind c-right = [end-word]
bind up = [move-up]
bind down = [move-down]
# Ctrl+Up/Down scroll the view; still owned by the text area.
bind c-up = [noop]
bind c-down = [noop]

# selection navigation
bind c-s-home = [select-document-begin]
bind c-s-end = [select-document-end]
bind s-pgup = [select-page-up]
bind s-pgdn = [select-page-down]
bind s-left = [select-left]
bind s-right = [select-right]
bind c-s-left = [select-word-left]
bind c-s-right = [select-word-right]
bind s-up = [select-up]
bind s-down = [select-down]

# line movement / reserved editor keys
bind a-up = [move-lines-up]
bind a-down = [move-lines-down]
# Ctrl+Alt+Up/Down (add cursor) is not implemented; left to the text area.
bind c-a-up = [noop]
bind c-a-down = [noop]

# clipboard / undo / redo
bind ins = [toggle-overtype]
bind s-ins = [paste]
bind c-ins = [copy]
bind c-a = [select-all]
bind c-l = [select-line]
bind c-c = [copy]
bind c-x = [cut]
bind c-v = [paste]
bind c-z = [undo]
bind c-y = [redo]
bind c-s-z = [redo]

# editor actions
bind a-j = [join-line]
bind a-z = [word-wrap]

# Every editor action above is a command, so you can rebind any key or bind a
# new one to it (see docs/macros.md for the full list). The few [noop] keys are
# handled by the text area itself: Ctrl+Up/Down scroll and Ctrl+Alt+Up/Down
# add a cursor.
