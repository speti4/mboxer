---
status: ready
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# Keep the attachment copies and the index's last-resort home in separate temp folders

What: `clean_temp` in `crates/mboxer-app/src/attachments.rs` empties the whole `<temp>/mboxer` folder at start-up,
and the core's last-resort index location (`fingerprint.rs`) is `<temp>/mboxer/index`, so those indexes are deleted
on every launch, possibly while another instance opens them; the per-attachment folder names (`{file}-{offset}-
{index}`) also repeat across processes, so two running instances overwrite each other's copies, and reopening an
attachment that is still open in its application fails on the locked file. Give the copies a per-process folder
under `<temp>/mboxer/attachments/`, clean only folders whose process is gone, leave `index/` alone, and reuse an
existing copy when its bytes match.
Why: a second instance or a read-only archive in a temp-backed index should not lose data at start-up.
Done when: two instances open the same attachment without an error, the index under `<temp>` survives a restart,
and the clean-up test covers a stale folder from a dead process.
