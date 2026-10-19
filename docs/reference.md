# mboxer — reference

The full behaviour of the viewer and the command line. For a first start, read the short guide in
[usage.md](usage.md), which also ships in the release zip.

## The viewer

`mboxer.exe` (or `cargo run -p mboxer-app -- [file.mbox]` during development) opens an mbox through a file
dialog or from its first argument. It builds the sidecar index on the first open, with progress and a cancel
button, and reuses it afterwards. Then it shows the archive in three panes: the Gmail label tree with counts,
the message list, and a reader with the headers and the body. The list shows the newest messages first and is
virtualised, so a 100 000-message archive scrolls like a short one.

Several mbox files can be open side by side (a split Takeout export, more than one export). Each is a row at
the top of the folder pane, one is active at a time, and the open set is remembered across starts (D-006).

### Labels

The folder pane shows the Gmail labels as a tree. `Work/Clients` sits under `Work`, the Takeout `Category …`
labels sit under *Categories*, and the system labels come first. Each label shows the number of messages that
carry it; a parent that only groups sub-labels shows their total in italics. An RFC 2047 encoded label name
(`=?UTF-8?Q?Sz=C3=A1ml=C3=A1k?=`) is decoded before it is indexed. Clicking a label narrows the list to that
label *and* every label under it; a chevron (or `ArrowLeft` / `ArrowRight` on the selected node) folds and
unfolds a branch.

### Conversations

By default the list shows one row per Gmail conversation (`X-GM-THRID`): the newest message with its subject,
a count badge and a chevron. The chevron, `ArrowRight` or a click opens the conversation under its row, oldest
message first; `ArrowLeft` closes it again, also from one of its messages. Only the messages that match the
current search belong to a conversation row, so a search can show a thread with fewer messages than it has; a
thread with one matching message is a plain row. The *Conversations* checkbox in the filter panel switches back
to one row per message; the choice is remembered. The status bar counts both: `N conversations · M messages`.

### Reader

An HTML message renders as HTML. The *HTML* / *Text* / *Source* buttons switch between the rendered body, the
plain-text part and the raw source of the message; HTML or Text is remembered. The HTML is sanitized in the
webview before it is shown (scripts, frames, forms, event handlers and `javascript:` links are removed) and
rendered in a sandboxed iframe whose own content security policy allows no network request at all. Remote
images and styles are held back and a yellow bar counts them; *Show* loads them for that one message. Links in
the body open in the system browser or mail client (`http`, `https`, `mailto` only). *Copy* puts the text of
the current view on the clipboard. In the Text view the mboxrd quoting of body lines (`>From `) is undone; the
Source view shows the bytes as they are in the file.

*Save .eml…* (`Ctrl+S`) writes the message as it sits in the mbox. *Print…* (`Ctrl+P`) opens the print
preview with the message laid out as the reader shows it, and a PDF printer turns it into a PDF. On a Mac the
printout opens in a print window of its own under the native print panel (*PDF › Save as PDF*); the window stays
until it is closed.

### Attachments

A strip under the headers lists the attachments: name, size, and an `inline` tag on images that belong to the
HTML body. A click on a chip decodes the attachment from the mbox into a temporary file under `%TEMP%\mboxer\`
and opens it with the default application. *Save…* asks where to put it, and *Save all…* writes every
attachment of the message into a folder without overwriting (`name (2).ext`). Executable attachments (`.exe`,
`.bat`, `.ps1`, `.js`, `.lnk` and the like) can only be saved. Every written file carries the Mark of the Web,
so Office opens it in Protected View, as with a file downloaded from a mail client; the temporary copies are
removed at the next start. Inline images referenced by `cid:` are shown inside the HTML body; text attachments
(csv, ics) are saved byte for byte in their original charset.

### Search

The field in the toolbar (`Ctrl+F` or `Ctrl+K`) narrows the list while you type. Words are AND-ed, the word you
are still typing counts as a prefix until you type a space, `"quotes"` make a phrase, `OR` and `-word` work,
and accents and case do not matter. The field also takes Gmail-style operators, and it is the one source of
truth: the *Filters* panel (`Ctrl+Shift+F`) and a click on a label in the folder pane write their operator into
the field, and an operator you type shows up in the panel and in the folder selection.

| Operator | Meaning |
|---|---|
| `from:anna`, `from:"anna példa"` | words in the sender (several `from:` add up) |
| `to:konyveles` | words in a recipient (To or Cc) |
| `subject:számla` | words in the subject |
| `after:2026-01-01`, `before:2026-01-31` | local days, both inclusive (`before:` includes the day, unlike Gmail; `2026/1/31` is accepted) |
| `has:attachment`, `-has:attachment` | with, without an attachment |
| `label:Inbox`, `label:"Work/Invoices"` | the first `label:` is the folder-pane node (it and its sub-labels), further ones are exact labels that must all be present; a container name (`label:Work`, `label:Categories`) selects the container |

Operator names are case-insensitive; `-` only negates `has:`. An unknown `foo:bar` is passed to the search
engine as is (it warns in the status bar); a bad date or `has:` value is a warning and the token is ignored.
The panel adds the sort order (newest first, or best match when there are words to score) and the
*Conversations* view. All of it is one query. The words you searched for (and the `from:` / `subject:` values)
are highlighted in the list and in the message body, the selected message stays selected when it is still in
the result, and *Clear* or `Esc` in the field empties the whole query, label included.

### Keyboard

| Key | Action |
|---|---|
| `Ctrl+O` | open an mbox |
| `Ctrl+F`, `Ctrl+K` | search field |
| `Ctrl+Shift+F` | filter panel |
| `Enter`, `ArrowDown` in the search field | move to the list |
| `Esc` in the search field | clear the query |
| arrows, `PageUp`, `PageDown`, `Home`, `End` | move in the list |
| `ArrowRight`, `ArrowLeft` | open, close a conversation or a label branch |
| `Enter` in the list | focus the reader; `Esc` returns |
| `Ctrl+S`, `Ctrl+P` | save the message as `.eml`, print it |

On a Mac `⌘` stands for `Ctrl` (the handler takes either key; the tooltips say `⌘`).

### Theme

The viewer follows the system colour mode; the button at the right end of the toolbar overrides it (*Auto*,
*Light*, *Dark*) and the choice is remembered. An HTML message body stays on a white page in dark mode, because
HTML mail is designed for one.

## The index

The index is a cache in `.mboxer\<file name>\` next to the mbox; delete it any time. It is rebuilt whole when
it no longer matches the file (D-004). When that folder is not writable (a read-only share, a DVD), the index
goes under `%LOCALAPPDATA%\mboxer\index\` (on a Mac `~/Library/Application Support/mboxer/index/`; D-005);
`MBOXER_INDEX_HOME` overrides that root, and every open looks in both places. Accents and case are folded on both the index and the query side, so `szamla` finds `Számla`.

## The command line

`mboxer-cli.exe` works on the same file and the same index, without a window.

```powershell
mboxer-cli scan <file.mbox> [--sample 3] [--limit N] [--quiet]
mboxer-cli index <file.mbox> [--force] [--threads N] [--quiet]
mboxer-cli search <file.mbox> [query words] [--from WORDS] [--to WORDS]
    [--subject WORDS] [--after YYYY-MM-DD] [--before YYYY-MM-DD] [--label NAME]... [--has-attachment | --no-attachment]
    [--limit 20] [--offset N] [--count] [--relevance]
```

`scan` streams the whole file once with a bounded buffer, prints the headers and MIME tree of the first
messages, then a summary: message count, throughput, largest message, labels, parse failures.

`index` builds the sidecar index and skips the build when the index already matches the file.

`search` needs that index. Query words are AND-ed, `"quotes"` make a phrase, accents and case are folded, the
filters narrow the result, and every hit line carries the message's byte offset and length in the mbox. Dates
are UTC days here, and `--before` is exclusive (unlike the viewer's `before:`).
