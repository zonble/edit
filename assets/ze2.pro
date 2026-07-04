# ze2.pro: sample macro profile, in the PE2/PE3 ".pro" style.
#
# A profile is just a list of commands, one per line, the same commands you can
# type in the command bar (Esc). Comment lines start with '#'. A macro is a
# named command sequence: "define <name> = [cmd] [cmd] ...", where each [cmd]
# is a command-bar command. Run a macro by typing its bare name, or "macro
# <name>". Built-in command names always win a collision, so a macro named
# "save" is only reachable as "macro save".
#
# Bind a key to a sequence with "bind <key> = [cmd] ...". Key names use the PE
# modifier prefixes c- (Ctrl), a- (Alt), s- (Shift), combinable and case-
# insensitive, over a base key: a letter/digit, a named key (enter, esc, tab,
# space, up, home, pgdn, del, ...), or f1-f12. A binding is checked before the
# built-in shortcuts, so it can override them.
#
# Run this file with "source <path>" in the command bar, or point the
# ZE2_PROFILE environment variable at it to load it automatically on startup.
# Macros and bindings live in memory for the session. An empty body removes a
# macro or binding (PE-style unbind): "define scratch =" / "bind c-j =".

# --- line editing -----------------------------------------------------------

# Duplicate the current line (PE3's [push mark][mark line][copy mark] idiom).
define dup-line = [mark-line] [copy-mark] [paste]

# Cut / copy a whole line without reaching for the mouse.
define cut-line  = [mark-line] [cut]
define copy-line = [mark-line] [copy-mark] [unmark]

# Uppercase / lowercase the current line in place.
define upper-line = [mark-line] [uppercase]
define lower-line = [mark-line] [lowercase]

# --- text templates ---------------------------------------------------------

# Insert a signature block. "insert" takes the rest of the token as literal text.
define sig = [insert -- ] [insert-line] [insert Jim Huang]

# Drop a date stamp on a fresh line.
define stamp = [insert-line] [insert Date: ] [date]

# A step may lead with a repeat count: "[N cmd]" runs cmd N times. Here: open
# five blank lines below.
define gap = [5 insert-line]

# --- formatting / composition ----------------------------------------------

# Reflow the paragraph, then save. A plain two-step macro.
define tidy = [reflow] [save]

# Macros call macros: "fmt" uppercases the line, then runs "tidy". This is the
# same mechanism as PE3's "[key <name>]". Recursion is capped, so a macro that
# (directly or indirectly) calls itself stops instead of looping forever.
define fmt = [macro upper-line] [macro tidy]

# --- editor setup -----------------------------------------------------------

# Turn on word wrap and the ruler for prose writing.
define prose = [word-wrap true] [toggle-ruler]

# --- key bindings -----------------------------------------------------------

# Bind keys to the macros above (and to inline sequences). These override the
# built-in shortcut for the same key.
bind c-d   = [macro dup-line]
bind a-u   = [macro upper-line]
bind f9    = [macro tidy]
bind c-s-p = [macro prose]

# A binding can also be an inline sequence with no named macro.
bind a-j   = [join-line]

# Bind "execute" to Ctrl-Space: run the current selection (or the current line)
# as a command sequence, turning the buffer into a macro scratchpad.
bind c-space = [execute]
