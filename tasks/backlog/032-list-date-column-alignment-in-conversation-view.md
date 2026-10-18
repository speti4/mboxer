---
status: idea
size: S
origin: wip/026-readme-front-page.md
created: 2026-10-18
---

# Date column misaligned between conversation rows and plain rows in the list

What: in the conversation view, a plain row (a thread with one message) ends its subject early and puts the date
right after it, while a conversation row puts the count badge and the date at the right edge, so the dates do not
line up (visible in `docs/images/viewer.png`). Give plain rows the same badge slot, or right-align the date column
for both. Why: the list is the most-read part of the window and a ragged date column looks broken.
Done when: dates align in one column in both views and in both themes.
