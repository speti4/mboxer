---
status: waiting
waits-for: every other open task done first (user decision 2026-10-19) — 043, and 040 run by a colleague with a Mac; 029 follows 028
size: S
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# Publication day: visibility switch and the public-only repo settings

What: a checklist done in one sitting: visibility to public; a ruleset on `main` (CI required, no force push,
linear history); private vulnerability reporting; secret scanning with push protection; a CodeQL workflow (Rust,
JavaScript for `ui/`, actions); build provenance attestation in the release job; topics, homepage and a social
preview image (from a synthetic screenshot); Discussions on or off; GitHub's Community Standards page checked
complete (023); the README badges render (026); the first release after the switch (the first end-to-end run of
the 025 release job: notes from `CHANGELOG.md`, `SHA256SUMS.txt`, the licence page in the zip; D-012); after the
switch, the pre-rewrite backup `E:\projects\mboxer-pre-rewrite-2026-10-18\` deleted (030, D-010).
Why: on the Free plan these settings exist only for public repos (branch protection returns 403 today), so they
cannot be prepared earlier than the switch itself.
Done when: every item is applied or recorded in the Log with the reason it was left out.
