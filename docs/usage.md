# mboxer — user guide

A viewer for a Gmail Takeout mbox archive on Windows 11, and on macOS as a preview. Nothing to install. Every
feature in detail, the full search syntax and the command line are in the reference:
https://github.com/speti4/mboxer/blob/main/docs/reference.md

## What you need

- Windows 11 (the viewer runs on the WebView2 that is part of Windows), or a Mac with macOS 10.13 or later,
  Apple Silicon or Intel. The macOS build is a preview: it has not yet been tested on a Mac.
- Your archive: the `.mbox` file from Google Takeout, anywhere on a disk you can read.
- Free space next to the mbox for the search index: about 1 % of the mbox size (a 14 GB
  archive gets a 135 MB index).

## Getting the program

The project's GitHub releases page (https://github.com/speti4/mboxer/releases) has one zip per platform. Each
zip holds the program, this guide as `usage.html` (it opens in your browser) and `THIRD-PARTY-LICENSES.html`,
the licences of the open-source parts compiled into it.

On Windows:

1. Download `mboxer-<version>.zip` and unzip it anywhere (next to the mbox, on the desktop, it does not
   matter). It holds `mboxer.exe`.
2. The exe is not code-signed, so Windows may show a SmartScreen warning on the first start:
   click *More info*, then *Run anyway*. If the exe refuses to start at all, right-click it,
   *Properties*, tick *Unblock*, *OK*.

On a Mac:

1. Download `mboxer-<version>-macos.zip` and double-click it in Finder. Move `mboxer.app` to *Applications*.
2. The app is not notarized by Apple, so macOS refuses the first start with a message that it cannot check the
   app. Open *System Settings → Privacy & Security*, scroll to the message about mboxer and click *Open Anyway*,
   then confirm. macOS asks only once.

To check a download, compare its SHA-256 with the value in `SHA256SUMS.txt` from the same release: in
PowerShell `(Get-FileHash mboxer-<version>.zip).Hash`, on a Mac `shasum -a 256 mboxer-<version>-macos.zip` in
Terminal (the case of the letters does not matter). With the GitHub CLI installed,
`gh attestation verify <zip> --repo speti4/mboxer` also shows that the project's own release workflow built
the zip.

## First open

Start mboxer and pick the mbox with *Open…* (`Ctrl+O`, `⌘O` on a Mac). On Windows you can also drag the mbox
file onto `mboxer.exe` in Explorer (a shortcut with `mboxer.exe "D:\mail\All mail.mbox"` does the same); on a
Mac, drop it on the app's Dock icon or use *Open With → mboxer* in Finder.

The first open builds the search index: a progress bar counts the messages, and the index of a
14 GB archive takes about two minutes on an SSD. *Cancel* stops it and leaves nothing behind.
Every later open reuses the index and takes well under a second. A new Takeout export is a new
file: its first open builds a new index.

The status bar at the bottom shows the path of the file on the left, the number of conversations and
messages in the list in the middle, and on the right "Index built in …" or "Index reused (built …)"; point at
that text to see the folder the index is in.

## Several files

Takeout splits a large archive into several `.mbox` files, and you may keep more than one export.
*Open…* adds to what is open instead of replacing it, and the dialog takes several files at once
(they are indexed one after the other). Every open file is a row at the top of the folder pane;
the one whose labels are shown is the active file, and the list, the search and the reader belong
to it. Click another file's row to switch (its own labels unfold, the previous file folds up); the
search you typed in a file waits for you when you come back to it, as long as mboxer runs. The `×` at
the right end of a row closes that file. There is no merged view: each file is searched on its own.

mboxer remembers the open files and which one was active. At the next start the ones whose index is
still fresh open by themselves; a file whose index would need a build stays greyed out in the pane until
you click it, and a file that is gone leaves the list. With no file open, the start screen has a
*Reopen …* button for every remembered file.

## Where mboxer writes

Everything below is a cache: deleting it costs a rebuild or the remembered settings, never mail.

| What | Windows | Mac |
|---|---|---|
| The search index | `.mboxer\<mbox file name>\` next to the mbox | `.mboxer/<mbox file name>/` next to the mbox |
| The index when that folder is not writable (a read-only share, a DVD) | `%LOCALAPPDATA%\mboxer\index\` | `~/Library/Application Support/mboxer/index/` |
| One small `.lock` file per mbox, so that two builds of it never run at once | `%LOCALAPPDATA%\mboxer\index\` | `~/Library/Application Support/mboxer/index/` |
| Attachments you open from a message (one folder per running mboxer, cleaned at a start once that mboxer has quit) | `%TEMP%\mboxer\attachments\` | `$TMPDIR/mboxer/attachments/` |
| The web view's own data: the remembered files and settings, a cache | `%LOCALAPPDATA%\io.github.speti4.mboxer\` | the folders named `io.github.speti4.mboxer` under `~/Library/` |

The index folder is named `.mboxer`: Windows shows it, the Mac Finder hides it, as it hides every name that
starts with a dot. The mbox itself is never modified. The remembered settings are the theme, the pane
widths, *HTML* or *Text*, the *Conversations* view and whether the *Filters* panel is open; the window
itself opens at its default size and place every time.

## Finding mail

The folder pane on the left shows your Gmail labels as a tree with counts; a click narrows
the list to that label and everything under it, and *All mail* shows everything again. The list
shows one row per conversation (newest first); the chevron or `ArrowRight` opens the conversation,
`ArrowLeft` closes it. The *Conversations* box in the *Filters* panel switches to one row per message.

The search field (`Ctrl+F` or `Ctrl+K`) narrows the list as you type:

- a message must contain every word you type; accents and case do not matter (`szamla` finds `Számla`)
- `"quotes"` make a phrase
- `OR` between two words finds either of them
- `-word` or `NOT word` leaves out the messages with that word
- a `*` at the end of a word finds every word that starts with it (`szaml*`)
- a colon is plain text unless it belongs to one of the operators below: `Re: számla` searches the two words

The words you searched for are highlighted in the list and in the message. The field also takes Gmail-style
operators, and the *Filters* panel (`Ctrl+Shift+F`) writes the same operators into the field:

| Operator | Meaning |
|---|---|
| `from:anna`, `from:"anna példa"` | words in the sender |
| `to:konyveles` | words in a recipient (To or Cc) |
| `subject:számla` | words in the subject |
| `after:2026-01-01`, `before:2026-01-31` | local days, both inclusive |
| `has:attachment`, `-has:attachment` | with, without an attachment |
| `label:Inbox`, `label:"Work/Invoices"` | the first one: a label and its sub-labels, as a click in the folder pane writes it; each further one: a label the message must carry as well, without its sub-labels |

The panel also sets the order of the list: *Newest first*, or *Best match* once there are words to score.
Its *Clear* button empties the field and puts the order back to *Newest first*.

Keyboard:

- `Ctrl+O` open a file, `Ctrl+F` or `Ctrl+K` the search field, `Ctrl+Shift+F` the *Filters* panel
- in the search field: `Enter` or `ArrowDown` moves to the list; `Esc` empties the field, label included, but
  keeps the order of the list (*Clear* resets that too); `Esc` in an empty field moves to the list
- in the list: arrows, `PageUp`, `PageDown`, `Home` and `End` move; `Enter` focuses the reader; `ArrowLeft`
  outside an open conversation moves to the folder pane
- in the reader: `Esc` or `ArrowLeft` returns to the list
- in the folder pane: `ArrowUp` and `ArrowDown` move between the labels (and onto another open file),
  `ArrowRight` unfolds a label or moves to the list, `ArrowLeft` folds it, `Enter` moves to the list
- `Ctrl+S` saves the message as `.eml`, `Ctrl+P` prints it

On a Mac `⌘` stands for `Ctrl` in every shortcut.

## Reading a message

The reader on the right shows the headers, the attachments and the body, with a bar of buttons over the body:

- *HTML*, *Text*, *Source*: the message as its sender laid it out, its plain text, or the whole message as it
  is in the mbox file. The choice of *HTML* or *Text* holds for the next messages; *Source* only for the one
  shown.
- *Copy* puts the text of the view you see on the clipboard.
- *Save .eml…* and *Print…*: see the next section.

Images and styles that an HTML message would load from the internet are blocked, so that opening a message
tells nobody that you read it. A yellow bar counts them, and *Show* loads them for that one message. A link
in a message opens in your browser or mail client.

The attachments are listed under the headers. A click opens one with its usual program, *Save…* next to it
asks where to keep it, and *Save all…* (shown when there are two or more) writes them all into a folder you
pick, next to files of the same name rather than over them. An attachment that the system would run rather
than open, such as a program, a script, an installer or a disk image (`.exe`, `.js`, `.msi`, `.iso`, `.app`,
`.pkg`, `.dmg`, `.sh` and the like), cannot be opened from mboxer, only saved; the reference lists every such
type. Every attachment or message that mboxer writes to disk is marked as downloaded from the internet, as a
mail client marks it.

## Saving, printing or exporting a message

Two buttons at the right end of the reader's view bar take the shown message out of the archive:

- *Save .eml…* (`Ctrl+S`) writes the message as it sits in the mbox, with the subject as the
  file name. The file opens in Outlook, Thunderbird or any mail client, attachments included.
- *Print…* (`Ctrl+P`) opens the print preview of Windows' web view with the message laid out
  as the reader shows it: the headers, the attachment names, then the body in the view you have
  selected (HTML with its inline images, Text or Source). Pick a printer, or *Save as PDF* /
  *Microsoft Print to PDF* as the destination to get a PDF file. Remote images blocked in the
  reader stay out of the printout; click *Show* first if you want them in.
  On a Mac (`⌘P`) the same printout opens in a print window of its own with the macOS print
  panel over it; *PDF › Save as PDF* in the panel writes a PDF. Close the print window when you
  are done; it also closes with the viewer.

## Light and dark

The viewer follows the system colour mode. The button at the right end of the toolbar overrides
it: it shows *Auto* (follows the system), *Light* or *Dark*, and a click moves to the next one; the
choice is remembered. The body of an HTML message stays on a white page in dark mode on purpose: HTML
mail is designed for a light page, and
a dark one would make some messages unreadable. The *Text* and *Source* views follow the theme.

## When something is off

- *Could not open the file*, with the path and the reason under it: the file is not an mbox ("no mbox
  envelope … at byte offset …"), or it cannot be read or does not exist any more. *Retry* tries the
  same file again, *Open another file…* picks a different one. When other files are open, the same
  reason shows in red in the status bar instead, as "Open failed: …". A remembered file that is gone
  leaves the pane at the next start; its `×` removes it at once.
- The reason is "another build holds …; wait for it to finish": the same mbox is being indexed
  elsewhere (another mboxer window, or `mboxer-cli`). Open it again when that build has finished.
- After a build the status bar says some messages were "too large to index": each is over 256 MB,
  which only a damaged file reaches. They are left out of the list and the search; the rest is indexed.
- The status bar shows "Query: …" in place of the index text: an operator has a value it cannot use
  (`after:` wants a day as `2026-01-31`, `has:` wants `attachment`). That operator is ignored until
  you fix it.
- A click on an attachment does nothing, and pointing at it says "executable attachments can only be
  saved": it is a type that would run as a program, so *Save…* is the way to get it (see *Reading a
  message*).
- The index rebuilds although nothing changed: the mbox was touched (copied with a new
  modification time, re-exported). A rebuild is safe, it only takes the time of the first open.
- To force a rebuild, delete the `.mboxer\<mbox file name>\` folder next to the mbox (or the
  folder under `%LOCALAPPDATA%\mboxer\index\`, on a Mac under `~/Library/Application Support/mboxer/index/`)
  and open the mbox again.
- On Windows, the window stays blank or does not open at all: the WebView2 runtime is missing or
  damaged. Install it from Microsoft (https://developer.microsoft.com/microsoft-edge/webview2/, the
  *Evergreen Runtime*) and start mboxer again.
- The version you run is at the right end of the status bar; the one on the releases page is
  the latest.
- To report a problem, open an issue at https://github.com/speti4/mboxer/issues. Describe it with
  made-up mail only: never attach or paste your own messages.
