---
status: idea
size: S
origin: done/058-dead-task-references-plan-slugs-and-milestone-closing.md
created: 2026-10-19
milestone: M6 Hardening
---

# Decide what the `plan:` field carries and whether closed files are rewritten

What: 43 task files carry a `plan:` value that is the bare name of a plan file outside the repo, and most of those
names are slugs derived from prompts (task 058 removed such names from the decisions' `sources:`, the `plan:` fields
stayed). Decide the rule for new tasks (`none`, or a neutral name chosen at promote time, never a prompt slug), write
it into CLAUDE.md "Tasks" and `tasks/README.md`, and decide whether the closed files get the same treatment in one
`tasks:` commit (the names carry no meaning for a reader of the public repo) or stay as history.
Why: the field is public text through the issue mirror, and a prompt slug is a trace of a conversation.
Done when: the rule is written, every open task follows it, and the closed files are either rewritten or named as
history in the rule.
