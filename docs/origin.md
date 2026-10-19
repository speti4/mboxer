# Origin

mboxer started as a greenfield project with one concrete problem behind it.

## Starting point

A Gmail mailbox archived with Google Takeout arrives as one large mbox file: about 14 GB in the reference case
(`All mail Including Spam and Trash.mbox`, with Takeout's settings files next to it). Years of mail in one file
are hard to use without a viewer built for that size.

## Problem

A viewer for such an archive has to:

1. open a file of tens of gigabytes without freezing, and re-open it fast,
2. search the message bodies in well under a second, which needs an index rather than a scan of the file,
3. put search first: one field with a hotkey, since finding a message is the most common task.

## Need

First, look for a free alternative (see `docs/research/`); the expectation was that none would fit. Second,
build an mbox viewer of its own for Windows 11 that meets the points above, with as few dependencies as
possible, ideally none for the person running it.

## Usage profile

Occasional lookups in an archived mailbox: opening a large archive has to be cheap, and finding a message has
to be fast and keyboard-friendly.
