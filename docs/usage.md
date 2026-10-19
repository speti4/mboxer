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

Start mboxer and pick the mbox with *Open* (`Ctrl+O`, `⌘O` on a Mac). On Windows you can also drag the mbox
file onto `mboxer.exe` in Explorer (a shortcut with `mboxer.exe "D:\mail\All mail.mbox"` does the same); on a
Mac, drop it on the app's Dock icon or use *Open With → mboxer* in Finder.

The first open builds the search index: a progress bar counts the messages, and the index of a
14 GB archive takes about two minutes on an SSD. *Cancel* stops it and leaves nothing behind.
Every later open reuses the index and takes well under a second. A new Takeout export is a new
file: its first open builds a new index.

## Several files

Takeout splits a large archive into several `.mbox` files, and you may keep more than one export.
*Open…* adds to what is open instead of replacing it, and the dialog takes several files at once
(they are indexed one after the other). Every open file is a row at the top of the folder pane;
the one whose labels are shown is the active file, and the list, the search and the reader belong
to it. Click another file's row to switch (its own labels unfold, the previous file folds up); the
search you typed in a file waits for you when you come back to it. The `×` at the right end of a
row closes that file. There is no merged view: each file is searched on its own.

mboxer remembers the open files. At the next start the ones whose index is still fresh open by
themselves; a file whose index would need a build stays greyed out in the pane until you click it,
and a file that is gone leaves the list.

## Where mboxer writes

Everything below is a cache: deleting it costs a rebuild, never data.

| What | Windows | Mac |
|---|---|---|
| The search index | `.mboxer\<mbox file name>\` next to the mbox | `.mboxer/<mbox file name>/` next to the mbox |
| The index when that folder is not writable (a read-only share, a DVD) | `%LOCALAPPDATA%\mboxer\index\` | `~/Library/Application Support/mboxer/index/` |
| One small `.lock` file per mbox, so that two builds of it never run at once | `%LOCALAPPDATA%\mboxer\index\` | `~/Library/Application Support/mboxer/index/` |
| Attachments you open from a message (cleaned at the next start) | `%TEMP%\mboxer\` | `$TMPDIR/mboxer/` |
| The web view's own data (window state, cache) | `%LOCALAPPDATA%\io.github.speti4.mboxer\` | the folders named `io.github.speti4.mboxer` under `~/Library/` |

The index folder is named `.mboxer`: Windows shows it, the Mac Finder hides it, as it hides every name that
starts with a dot. The mbox itself is never modified.

## Finding mail

The folder pane on the left shows your Gmail labels as a tree with counts; a click narrows
the list to that label and everything under it. The list shows one row per conversation
(newest first); the chevron or `ArrowRight` opens the conversation, `ArrowLeft` closes it.
The reader on the right shows the message with its attachments (click to open, *Save…* to keep).

The search field (`Ctrl+F`) narrows the list as you type. A message must contain every word you
type, `"quotes"` make a phrase, `OR` and `-word` work, accents and case do not matter (`szamla` finds `Számla`). It
also takes Gmail-style operators, and the *Filters* panel (`Ctrl+Shift+F`) writes the same
operators into the field:

| Operator | Meaning |
|---|---|
| `from:anna`, `from:"anna példa"` | words in the sender |
| `to:konyveles` | words in a recipient (To or Cc) |
| `subject:számla` | words in the subject |
| `after:2026-01-01`, `before:2026-01-31` | local days, both inclusive |
| `has:attachment`, `-has:attachment` | with, without an attachment |
| `label:Inbox`, `label:"Work/Invoices"` | a label and its sub-labels |

Keyboard: `Ctrl+O` open, `Ctrl+F` search, `Esc` in the field clears the whole query, `Enter`
or `ArrowDown` moves to the list, arrows / `PageUp` / `PageDown` / `Home` / `End` move in the
list, `Enter` focuses the reader, `Esc` returns to the list, `Ctrl+S` saves the message as
`.eml`, `Ctrl+P` prints it. On a Mac `⌘` stands for `Ctrl` in every shortcut.

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
it: *Auto* (follows the system), *Light*, *Dark*; the choice is remembered. The body of an HTML
message stays on a white page in dark mode on purpose: HTML mail is designed for a light page, and
a dark one would make some messages unreadable. The *Text* and *Source* views follow the theme.

## When something is off

- *Open failed* in red: the file is not an mbox (the message names the byte offset), or the path
  does not exist any more. Pick the file again with *Open*; a file that is gone can be removed from
  the pane with its `×`.
- *Open failed* with "another build holds …": the same mbox is being indexed elsewhere (another
  mboxer window, or `mboxer-cli`). Open it again when that build has finished.
- The status bar says some messages were "too large to index": each is over 256 MB, which only a
  damaged file reaches. They are left out of the list and the search; the rest is indexed.
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
