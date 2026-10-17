---
status: proposed
date: 2026-10-17
cost: one-way
sources: [wip/002-rust-workspace-and-mbox-stream-parser.md, folytassuk-a-002-task-wise-starfish.md]
---

# D-03 — A message is identified by the byte offset of its `From ` envelope line

## Context

The sidecar index (003) and the viewer must find a message in a 14–50 GB mbox without rescanning it.
The streaming reader (002) is the only component that walks the file, so the identity it hands out
is what every later layer stores. Changing it afterwards would invalidate every index already built
on users' machines, so the contract is fixed before the first index exists.

## Decision

- `offset` = the byte position of the first byte of the message's `From ` envelope line.
- `len` = bytes from that position up to (not including) the next envelope line, or to the end of
  the file for the last message. The envelope line is part of the range.
- `read_at(offset, len)` reproduces exactly the bytes the streaming pass yielded and re-validates the
  envelope line; a mismatch means the mbox changed and the index must be rebuilt.
- Envelope lines are recognised by shape (`From ` + token + weekday + month + day + `hh:mm:ss` …
  4-digit year as the last field), not by a Gmail-specific pattern, so other mbox producers work.
- `>From ` lines inside bodies are left as written (no mboxo/mboxrd unescaping) until a reader
  feature needs it; search is unaffected.

Proposed by the session on 2026-10-17; awaiting the user's confirmation.

## Not chosen

- `Message-ID` as identity: not unique (duplicates, missing in some messages) and needs parsing before seeking.
- Sequence number (nth message): any change before the message shifts every later id; a seek still needs offsets.
- Offset of the first header line (after the envelope): loses the envelope date and label-free re-validation.
- Content hash: costs a full read per lookup and still needs an offset to seek.
