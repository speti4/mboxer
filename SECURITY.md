# Security policy

mboxer opens mail archives that hold years of private correspondence, and it renders HTML that strangers wrote.
A flaw in how it does that matters, and a report is welcome.

## Supported versions

Only the latest release on the [Releases page](https://github.com/speti4/mboxer/releases) gets fixes. A fix
ships as a new release; there are no backports.

## Reporting a vulnerability

Use GitHub's private vulnerability reporting:
[Report a vulnerability](https://github.com/speti4/mboxer/security/advisories/new) (the *Security* tab →
*Report a vulnerability*). Please do not open a public issue for a security problem.

Never attach real mail to a report, yours or anyone else's. Describe the problem with a made-up message: a
few lines of mbox with `example.com` addresses are enough to reproduce almost anything (see the synthetic
fixtures under `crates/mboxer-core/tests/fixtures/`).

You can expect a first answer within a week. Once a fix is released, the advisory is published with credit to
you unless you ask otherwise.

## What is in scope

The places where mboxer handles untrusted input or writes to disk:

- **The HTML reader.** Message HTML is sanitized in the webview before it is shown: scripts, frames, forms,
  event handlers and `javascript:` links are removed. It renders in a sandboxed iframe whose own content
  security policy allows no network request. Remote images and styles stay blocked until the user clicks
  *Show* for that one message. A way to run script, reach the network without *Show*, or get out of the
  sandbox is a vulnerability.
- **Links.** Links in a message open in the system browser or mail client, and only for `http`, `https` and
  `mailto`. Any other scheme getting through is a vulnerability.
- **Attachments.** An attachment opened from a message is decoded into `%TEMP%\mboxer\` (on macOS
  `$TMPDIR/mboxer/`) and handed to its default application. Every file mboxer writes is marked as downloaded
  from the internet: the Mark of the Web on Windows (`Zone.Identifier`, internet zone), so Office opens it in
  Protected View, and the `com.apple.quarantine` attribute on macOS. Executable types (`.exe`, `.bat`, `.ps1`,
  `.js`, `.lnk` on Windows; `.app`, `.command`, `.pkg`, `.dmg`, `.sh`, scripts and internet-location files on
  macOS; and the like) can only be saved, never opened. A written file without the mark, an executable that
  opens, or a file name that writes outside the chosen folder is a vulnerability.
- **The parser and the index.** The mbox is read with a bounded buffer and is never modified. The sidecar
  index lives in `.mboxer\<file name>\` beside the mbox, or, when that folder is not writable, under
  `%LOCALAPPDATA%\mboxer\index\` (on macOS `~/Library/Application Support/mboxer/index/`). A crafted mbox that crashes the viewer, exhausts memory, or makes mboxer write
  outside those places is in scope. So is a crafted index folder that does more than force a rebuild.
- **The application shell.** The Tauri capability file and the window's content security policy
  (`crates/mboxer-app/capabilities/`, `crates/mboxer-app/tauri.conf.json`).

## Out of scope

- The exe is not code-signed, so SmartScreen warns on the first start (see `docs/usage.md`).
- The macOS app is ad-hoc signed and not notarized, so Gatekeeper blocks its first start until it is allowed
  under *Privacy & Security* (see `docs/usage.md`).
- Anything that needs an attacker who can already write to the user's disk or run code as the user.
- Bugs in WebView2 or Windows itself, or in WKWebView or macOS. Report those to Microsoft or Apple. A way
  mboxer could defend against one is welcome as an ordinary issue.
