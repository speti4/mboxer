# Origin

mboxer started as a greenfield project with one concrete problem behind it.

## Starting point

A former work mailbox lived in Gmail and was archived with Google Takeout: one mbox file of about 13 GB
(`All mail Including Spam and Trash.mbox`, with Takeout's settings files next to it). Until now that archive
was opened with the open-source [eneam/mboxviewer](https://github.com/eneam/mboxviewer).

## Problem

That viewer:

1. has a poor UI and UX,
2. is unstable and slow, and often freezes,
3. hides search behind an awkward "Advanced Find" dialog with no hotkey, which makes the most common task the
   least comfortable one.

## Need

First, look for a free alternative (see `docs/research/`); the expectation was that none would fit. Second,
build an own mbox viewer for Windows 11 that solves the problems above, with as few dependencies as possible,
ideally none for the person running it.

## Usage profile

Occasional lookups in an old mailbox, once or twice a week at most, no heavy use: opening a large archive has
to be cheap, and finding a message has to be fast and keyboard-friendly.
