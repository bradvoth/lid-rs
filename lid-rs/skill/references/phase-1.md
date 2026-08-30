# Phase 1 — LLD (human-owned; you draft)

Plain English in `docs/intent/<slice>/lld.md`, wired into the module:

```rust
#[doc = include_str!("../../docs/intent/<slice>/lld.md")]
pub mod slice_name;
```

This layer cannot be recovered from code: rationale, rejected alternatives,
invariants that aren't type-expressible. Write the Context and Design
Philosophy, the Behaviour, and a Decisions & Alternatives table (decision,
chosen, alternatives considered, rationale) — a decision with no alternative
considered is a decision not yet examined.

Rust code blocks in an LLD become live doctests through the include — write
them compilable (hide setup lines with `# `) or the doc gate fails.

Docs are written fresh-author: no narration of how they changed ("we now
also..."), no meaning that needs this conversation to parse. A reader who
has never seen the change should get the same LLD as one who watched it
happen.

For a Phase 8 change to an existing slice, this phase is "edit the LLD" —
same fresh-author discipline, same STOP, but see `phase-8.md` for the
cascade that follows.

`cargo lid-rs coach` runs this phase as an interview instead: a session that
reads the repository, questions the human one question at a time, drafts the
document, and has it read back by the checks and the reader between drafts.
Its method is `coach.md`; what it produces is a draft, and approving and
committing it are still the human's.

**STOP for review.** Commit as `phase 1: LLD for <slice>` once approved.
