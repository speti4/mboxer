# mboxer — reference

The full behaviour of the viewer and the command line. For a first start, read the short guide in
[usage.md](usage.md), which also ships in the release zip as `usage.html`.

## The viewer

`mboxer.exe` on Windows, `mboxer.app` on macOS (or `cargo run -p mboxer-app -- [file.mbox]` during development)
opens an mbox through a file dialog or from its first argument; on macOS also from Finder's *Open With* and a
drop on the Dock icon. It builds the sidecar index on the first open, with progress and a cancel
button, and reuses it afterwards. Then it shows the archive in three panes: the Gmail label tree with counts,
the message list, and a reader with the headers and the body. The list shows the newest messages first and is
virtualised, so a 100 000-message archive scrolls like a short one.

While a file opens, an overlay says "Checking the index…" (only when the check takes longer than a moment) and,
during a build, "Building the search index…" with the bytes read, the messages counted and the speed in MB/s; with
several files it counts them ("file 2 of 3"). A file that cannot be opened while no other file is open shows a
screen titled "Could not open the file" with the path, the reason and two buttons, *Retry* and *Open another
file…*; while other files are open, the reason goes to the status bar as "Open failed: …" instead. The reasons
come from the reader and the index: "no mbox envelope (`From ` line) at byte offset N" for a file that is not an
mbox, an I/O error for a file that is gone or unreadable, and "another build holds `<lock file>`; wait for it to
finish" (see [The index](#the-index)).

Several mbox files can be open side by side (a split Takeout export, more than one export). Each is a row at
the top of the folder pane, one is active at a time, and the open set is remembered across starts
([D-006](../tasks/decisions/D-006-several-files-open-one-active-no-merged-view.md)). *Open…* adds files to
the set, several per dialog, opened one after the other. A click on another file's row makes it active, a
click on the active file's row selects its *All mail*, and the row's `×` closes the file and forgets it. The
query of a file that is not active (the field text and the sort order) waits for its next turn while the
viewer runs; it is not saved.

At a start the remembered files whose index is fresh open again, the previously active one first, so it is
active again; a file whose index needs a build stays in the pane in grey italics until it is clicked (its
`×` only forgets it), and a file that no longer exists is dropped with "1 remembered file is gone" (or "N
remembered files are gone") in the status bar. A file named on the command line or handed over by Finder
opens after them and becomes the active one. With no file open, the start screen offers *Open mbox files…*
and a *Reopen <file name>* button for each remembered file.

The viewer keeps its settings in the web view's local storage (under `%LOCALAPPDATA%\io.github.speti4.mboxer\`
on Windows, the `io.github.speti4.mboxer` folders under `~/Library/` on a Mac): the remembered files and the
active one, the theme, the widths of the folder pane and the list (drag the splitters between the panes), *HTML*
or *Text*, the *Conversations* view and whether the filter panel is open. The window's size and position are not
saved: it opens centred at 1400 × 900.

### Labels

The folder pane shows the Gmail labels as a tree. `Work/Clients` sits under `Work`, the Takeout `Category …`
labels sit under *Categories*, and the system labels come first. Each label shows the number of messages that
carry it; a parent that only groups sub-labels shows their total in italics. An RFC 2047 encoded label name
(`=?UTF-8?Q?Sz=C3=A1ml=C3=A1k?=`) is decoded before it is indexed. Clicking a label narrows the list to that
label *and* every label under it, and *All mail* lifts the label again; a chevron (or `ArrowLeft` /
`ArrowRight` on the selected node) folds and unfolds a branch. With the focus in the pane, `ArrowUp`,
`ArrowDown`, `Home` and `End` move the selection, and the list follows at once (an arrow onto another loaded file's row switches to
that file; a file that is not loaded is skipped, so an arrow never starts a build), `ArrowRight` on a folded
node unfolds it and on any other node moves to the list, as `Enter` does, and `ArrowLeft` folds an unfolded node.

### Conversations

By default the list shows one row per Gmail conversation (`X-GM-THRID`): the newest message with its subject,
a count badge and a chevron. A click on the chevron or `ArrowRight` opens the conversation under its row, oldest
message first; `ArrowLeft` closes it again, also from one of its messages. Only the messages that match the
current search belong to a conversation row, so a search can show a thread with fewer messages than it has; a
thread with one matching message is a plain row. The *Conversations* checkbox in the filter panel switches back
to one row per message; the choice is remembered. The status bar counts both: `N conversations · M messages`.

### Reader

The reader shows the subject, the From, To, Cc, Date, Labels and Message-ID headers, the attachments, the body,
and at the bottom a folded *MIME parts* list (each part's type, transfer encoding, name and size).

An HTML message renders as HTML. The *HTML* / *Text* / *Source* buttons switch between the rendered body, the
plain-text part and the raw source of the message; HTML or Text is remembered and applies to the next messages,
Source holds for the shown message only. *HTML* is disabled for a message without an HTML part; a message with
only an HTML part shows text made from that HTML in the Text view, and a message that could not be parsed shows
its raw text. The Source view shows the envelope (`From `) line and the message bytes as they are in the file,
decoded as UTF-8. The HTML is sanitized in the webview before it is shown (scripts, frames, forms, event
handlers and `javascript:` links are removed) and rendered in a sandboxed iframe whose own content security
policy allows no network request at all. Remote images and styles are held back and a yellow bar counts them
("3 remote images or styles blocked"); *Show* loads them for that one message, without a referrer. Links in the
body open in the system browser or mail client (`http`, `https`, `mailto` only).
*Copy* puts the text of the current view on the clipboard (the rendered text of the HTML view, the body of the
Text view, the whole source in the Source view) and says "Copied" in the status bar. In the Text view the mboxrd
quoting of body lines (`>From `) is undone; the Source view shows the bytes as they are in the file.

*Save .eml…* (`Ctrl+S`) writes the message as it sits in the mbox, with the mboxrd quoting undone and the
subject as the suggested file name; the status bar names the saved file. *Print…* (`Ctrl+P`) lays out a
print document: the subject, the From, To, Cc, Date and Labels headers, the attachment names with their sizes,
then the body in the current view (the HTML view with its inline images, and its remote images only when *Show*
has loaded them; the Text or Source view as plain text). On Windows it opens the web view's print preview, where
a PDF printer (*Save as PDF*, *Microsoft Print to PDF*) turns it into a PDF. On a Mac the printout opens in a
print window of its own under the native print panel (*PDF › Save as PDF*); the window stays until it is closed
and closes with the viewer. `Ctrl+P` without a message does nothing (it never prints the viewer's own window).

### Attachments

A strip under the headers lists the attachments: name, size, and an `inline` tag on images that belong to the
HTML body. A click on a chip decodes the attachment from the mbox into a temporary file under
`%TEMP%\mboxer\attachments\<process id>\` (on a Mac `$TMPDIR/mboxer/attachments/<process id>/`) and opens it with
the default application. *Save…* asks where to put it, and *Save all…* writes every attachment of the message
into a folder without overwriting (`name (2).ext`). Attachments
that the system would run instead of open can only be saved: their chip is disabled and its tooltip says
"executable attachments can only be saved". The Windows list: programs, scripts, shortcuts and installers
(`.exe`, `.bat`, `.cmd`, `.com`, `.scr`, `.pif`, `.msi`, `.msp`, `.ps1`, `.psm1`, `.vbs`, `.vbe`, `.js`, `.jse`,
`.wsf`, `.wsh`, `.hta`, `.lnk`, `.reg`, `.jar`, `.cpl`, `.inf`, `.url`, `.application`, `.gadget`, `.msc`,
`.appx`), script components (`.wsc`, `.sct`), packages (`.msix`, `.msixbundle`, `.appinstaller`), disk images
and archives that Explorer mounts (`.iso`, `.img`, `.vhd`, `.vhdx`, `.cab`), compiled help (`.chm`), Excel
add-ins (`.xll`) and shell command, settings, library and search files (`.scf`, `.settingcontent-ms`,
`.library-ms`, `.search-ms`). The Mac list: scripts, installers, disk images, bundles, Terminal settings and
internet-location files (`.command`, `.tool`, `.pkg`, `.mpkg`, `.dmg`, `.app`, `.sh`, `.scpt`, `.applescript`,
`.workflow`, `.terminal`, `.webloc`, `.inetloc`, `.fileloc`), the other shells' scripts (`.zsh`, `.bash`, `.csh`),
configuration profiles (`.mobileconfig`), preference panes (`.prefpane`) and Java Web Start files (`.jnlp`). Both
lists apply on both systems, since an attachment is as dangerous on the machine it is forwarded to. *Save all…*
shows when a message lists two or more attachments; the status bar then says how many files went into which
folder.
Every written file is marked as downloaded from the internet, as a mail client marks it: the Mark of the Web on
Windows, so Office opens it in Protected View, and the `com.apple.quarantine` attribute on a Mac, so Gatekeeper
checks it before it opens. A second click reuses the copy when its bytes are unchanged, so a file still open in
its application opens again. Each running mboxer keeps a lock file in its own folder; a later start removes the
folders whose mboxer has quit and never touches the last-resort index home beside them (`mboxer/index`). Inline
images referenced by `cid:` are shown inside the HTML body; text attachments (csv, ics) are saved byte for byte in
their original charset.

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

Operator names are case-insensitive; `-` only negates `has:`. Anything else with a colon is plain text:
`Re: számla` searches the words `re` and `számla`, and a word with punctuation inside (`re:számla`, `10:30`,
a URL, an unknown `foo:bar`) matches its parts in that order, like a phrase. The rest of the free-text syntax
is `OR` between two words (it binds tighter than the implicit AND: `számla ebéd OR március` is `számla` with
either of the others), `-word`, `-"a phrase"` or `NOT word` to exclude (an exclusion alone lists every other
message), and a trailing `*` for a prefix (`szaml*`); brackets, braces, `^`, `~` and the like are
punctuation. A bad date or `has:` value is a warning and the token is ignored: the status bar shows it as
"Query: after: expects a day as YYYY-MM-DD" or "Query: has: expects `attachment`" until the text is fixed.

The filter panel (*Filters* or `Ctrl+Shift+F`; its open or closed state is remembered) has *From*, *Date from*
and *to*, *Attachments* (*Any*, *With attachments*, *Without*), each of which writes its operator into the field,
and two settings beside the text: the *Conversations* view and *Sort* (*Newest first*, or *Best match*, which
applies only while there are words to score). All of it is one query. A badge on the *Filters* button counts the
active filters (`from:`, `to:`, `subject:`, the dates, `has:`, every `label:` after the first, a sort other
than *Newest first*). The words you searched for (and the `from:` / `to:` / `subject:` values) are highlighted
in the list, the reader's subject and the message body (a word with punctuation or a phrase is marked where its
parts stand in that order, so `re:számla` marks `Re: számla`; a body over 2 million characters is shown without
highlighting), and the selected message stays selected when it is still in the result.

`Esc` in the field empties the text, label included, and keeps the sort order; in an empty field it moves to the
list. *Clear* in the panel empties the text and sets the sort back to *Newest first*. Neither touches the
*Conversations* view, which is a setting.

### Keyboard

| Key | Action |
|---|---|
| `Ctrl+O` | open one or more mbox files |
| `Ctrl+F`, `Ctrl+K` | search field, its text selected |
| `Ctrl+Shift+F` | show or hide the filter panel |
| `Enter`, `ArrowDown` in the search field | move to the list |
| `Esc` in the search field | empty the text, label included (the sort order stays); in an empty field, move to the list |
| `Esc` in the filter panel, `Enter` in its *From* field | move to the list |
| `ArrowUp`, `ArrowDown`, `PageUp`, `PageDown`, `Home`, `End` in the list | move the selection |
| `ArrowRight` in the list | open a closed conversation; elsewhere move to the reader |
| `ArrowLeft` in the list | close the open conversation (also from one of its messages); elsewhere move to the folder pane |
| `Enter` in the list | focus the reader |
| `Esc`, `ArrowLeft` in the reader | return to the list |
| `ArrowUp`, `ArrowDown`, `Home`, `End` in the folder pane | select a label (or switch to another loaded file) |
| `ArrowRight`, `ArrowLeft` in the folder pane | unfold, fold a label branch; `ArrowRight` on an unfolded or leaf node and `Enter` move to the list |
| `Ctrl+S`, `Ctrl+P` | save the message as `.eml`, print it |

The `Ctrl` shortcuts work from anywhere in the window, the message body included. On a Mac `⌘` stands for
`Ctrl`: both keys work, and the tooltips, the search placeholder and the start screen show `⌘` (`⇧⌘F` for the
filter panel).

### Theme

The viewer follows the system colour mode; the button at the right end of the toolbar overrides it and the
choice is remembered. The button shows the current mode (*Auto*, *Light*, *Dark*), a click moves to the next one
in that order, and its tooltip names the current and the next mode ("Theme: follows the system. Click for
light."). The window's title bar follows the choice where the system allows it. An HTML message body stays on a
white page in dark mode, because HTML mail is designed for one; the Text and Source views follow the theme.

### Status bar

From left to right: the path of the active file; the size of the result, `N conversations · M messages` (or
`M messages` without the *Conversations* view, `M of T messages` while a search narrows the list); the index
state; the version (`v<version>`). The index state is "Index reused (built <date and time>)" or "Index built in N s,
K unparsed messages", with ", J too large to index" when a build skipped messages; its tooltip names the index
folder. The same place shows a query warning ("Query: …", until the text is fixed) and, for a few seconds, the
result of an action: "Saved <path>", "N files saved to <folder>", "Copied", "Open cancelled", "Open failed: …"
(in red, when another file stays open), "N remembered files are gone", or the error of a failed save, copy,
print or link.

## The index

The index is a cache in `.mboxer\<file name>\` next to the mbox; delete it any time. It is rebuilt whole when
it no longer matches the file ([D-004](../tasks/decisions/D-004-sidecar-index-location-and-freshness.md)). When
that folder is not writable (a read-only share, a DVD), the index goes under `%LOCALAPPDATA%\mboxer\index\` (on
a Mac `~/Library/Application Support/mboxer/index/`;
[D-005](../tasks/decisions/D-005-index-fallback-under-localappdata.md));
`MBOXER_INDEX_HOME` overrides that root, and every open looks in both places. Accents and case are folded on both the index and the query side, so `szamla` finds `Számla`.

A build writes into `<file name>.building\` beside the index and puts it in place only when it is complete; a
viewer may keep the old index open meanwhile. The old index's files are moved aside into `<file name>.old\` first
and deleted last, so a failed swap leaves the old index whole; a `.old` folder that a viewer still holds is deleted
by the next build. One build of a file runs at a time: a second one, from the viewer or `mboxer-cli`, stops at once
with "another build holds `<lock file>`". The lock file is `<file name>-<hash>.lock` in the index home above;
the system releases the lock when the build ends or its process dies, so a stale lock does not exist. A message
larger than the size limit (256 MB, which only a corrupt file reaches) is left out of the index and counted as
skipped, and the build goes on with the next message; a file of nothing but such text is still an error.

## The command line

`mboxer-cli` (`mboxer-cli.exe` on Windows) works on the same file and the same index, without a window. It is a
developer tool and not part of the release zips: build it with `cargo build --release -p mboxer-cli`.

```powershell
mboxer-cli scan <file.mbox> [--sample 3] [--limit N] [--quiet]
mboxer-cli index <file.mbox> [--force] [--threads N] [--quiet]
mboxer-cli search <file.mbox> [query words] [--from WORDS] [--to WORDS]
    [--subject WORDS] [--after YYYY-MM-DD] [--before YYYY-MM-DD] [--label NAME]... [--has-attachment | --no-attachment]
    [--limit 20] [--offset N] [--count] [--relevance]
```

`scan` streams the whole file once with a bounded buffer, prints the headers and MIME tree of the first
messages, then a summary: message count, throughput, largest message, labels, parse failures.

`index` builds the sidecar index and skips the build when the index already matches the file. Its summary counts
the skipped messages and lists the byte offsets of the first ten.

`search` needs that index. The query words follow the viewer's free-text syntax (AND-ed, `"quotes"`, `OR`,
`-word`, a trailing `*`, a colon is plain text; a query that starts with `-` goes after `--`), accents and
case are folded, the
filters narrow the result, and every hit line carries the message's byte offset and length in the mbox. Dates
are UTC days here, and `--before` is exclusive (unlike the viewer's `before:`).
