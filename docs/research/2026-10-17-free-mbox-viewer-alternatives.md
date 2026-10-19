# Free mbox viewer alternatives — survey (2026-10-17)

Question: is there a free Windows GUI that opens a 14–50 GB Gmail Takeout mbox and searches message
bodies fast? Answer: no. Verdict recorded in `tasks/decisions/D-001-build-own-viewer-instead-of-buying.md`.

| Tool | Type | Large files | Body search | Free? | Fit for this use |
|---|---|---|---|---|---|
| Mbox Viewer (mboxviewerpro.com, MS Store) | native GUI | 50 GB+ streaming, 1 MB buffer, re-open < 1 s | streaming scan, not indexed | free ≤ 1 GB, Pro paid (one-time or yearly) | free tier below this archive size; body search not indexed |
| mboxShell (dcarrero, Rust, MIT) | terminal TUI | 50 GB+, ~500 MB RAM, bincode sidecar index | metadata indexed (< 200 ms per 1M msgs); body = streaming scan | yes | no GUI; good implementation reference |
| zoltanf/MboxViewer (Electron, GPL-3) | desktop GUI | SQLite sidecar index | indexed | yes | no confirmed Windows build; Electron runtime |
| Thunderbird + ImportExportTools NG | mail client import | stalls reported on 60–89 GB | Gloda index after import | yes | an import into a mail client, not a viewer |
| CoolUtils / SysTools / Aryson / BitRecover "free viewers" | proprietary | not verified | not verified | freemium | claims not verified |

## Sources

- https://mboxviewerpro.com/ and https://mboxviewerpro.com/mboxshell/
- https://github.com/dcarrero/mboxshell
- https://github.com/zoltanf/MboxViewer
- https://support.mozilla.org/ca/questions/1338775 (Thunderbird large mbox stalls)
- https://www.coolutils.com/MboxViewer, https://windowsutils.com/blog/best-free-mbox-viewer-software-tools-review

## Takeaways for the build

- Streaming with a bounded buffer and a sidecar index is the proven pattern (mboxShell, Mbox Viewer).
- Nobody free indexes bodies on this scale; that is the gap mboxer fills (D-002).
- mail-parser (Rust, 5 light deps) handles MIME, charsets, attachments; mbox splitting is ours.
