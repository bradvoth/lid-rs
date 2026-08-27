# Phase 0 — Name the slice

Name it as a user-visible operation ("user logs in"), never a component
("auth module"). One LLD, one module boundary — if the name needs "and" to
say, it's two slices.

Create the working branch now, named for the slice: `lld/<slice-name>`. See
the main `SKILL.md`'s "Working state" section for what happens on it.

A waiver of any rule (e.g. skipping a fork's Phase 5 red-confirmation) is
per slice and must be restated here if it still applies — it does not carry
over from a previous slice, and a directive to a fork may not waive Phase 5
regardless.

Bug fixes name the slice the same way: the user-visible behavior that was
supposed to happen, not "fix bug #123."
