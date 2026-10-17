---
status: idea
size: S
origin: session 2026-10-17
created: 2026-10-17
milestone: M3 Comfort
---

# Portable release build for other users

What: Release profile, single portable exe (no installer), version stamp, a short `docs/usage.md` for the other 2–5 users; CI build optional.
Why: The other users must only copy one file next to their own mbox; nothing to install (D-02).
Done when: A colleague opens their own mbox on a clean Windows 11 machine with the exe alone.
Note from 005: the exe is `crates/mboxer-app` (`cargo build --release` → `target\release\mboxer.exe`, release = windowed, no console); WebView2 keeps its profile under `%LOCALAPPDATA%\io.github.speti4.mboxer` (the `identifier` in `tauri.conf.json`), which the usage doc should mention as the one thing the app writes outside the mbox folder. Consider `strip = true` / `opt-level` for size here, not earlier.
