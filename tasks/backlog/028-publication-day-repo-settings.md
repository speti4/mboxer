---
status: waiting
waits-for: 027 decided and 023-026 done
size: S
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# Publication day: visibility switch and the public-only repo settings

What: a checklist done in one sitting: visibility to public; a ruleset on `main` (CI required, no force push,
linear history); private vulnerability reporting; secret scanning with push protection; a CodeQL workflow (Rust,
JavaScript for `ui/`, actions); build provenance attestation in the release job; topics, homepage and a social
preview image (from a synthetic screenshot); Discussions on or off; the first release after the switch.
Why: on the Free plan these settings exist only for public repos (branch protection returns 403 today), so they
cannot be prepared earlier than the switch itself.
Done when: every item is applied or recorded in the Log with the reason it was left out.
