---
status: ready
size: S
origin: session 2026-10-18
created: 2026-10-18
milestone: M4 macOS
---

# macOS runtime differences: index home, quarantine, Finder open, UI keys and fonts

What: `default_index_home` falls back to `$HOME/Library/Application Support/mboxer/index` on macOS (D-05 amended,
CLI help and doc strings follow); `mark_of_the_web` gets a macOS arm writing the `com.apple.quarantine` xattr
(best effort, the `xattr` command, no new crate); `BLOCKED_EXTENSIONS` adds the macOS executable types (`command
tool pkg mpkg dmg app sh scpt applescript workflow terminal webloc inetloc fileloc`); `RunEvent::Opened` (Finder
"Open with", Dock drop) feeds `open_mbox`; the UI's path keys (`files.js`, `main.js queryKey`) become OS-aware,
tooltips show `⌘` on a Mac, the monospace stack becomes `ui-monospace, Menlo, Consolas, monospace`, "follows
Windows" becomes "follows the system"; `files.test.js` takes the OS as a parameter.
Why: the Windows assumptions the 2026-10-18 sweep found outside printing (020) and packaging (018).
Done when: `cargo test` and the node tests pass on both CI runners; on a Mac an opened attachment carries the
quarantine xattr, a read-only mbox folder indexes under `~/Library`, and Finder "Open with" opens the file.
