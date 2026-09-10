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

**Spike before you specify a mechanism.** An LLD may assert what the system
shall do on the author's judgment. It may not assert what a *tool* can do — a
macro reading something, the compiler rejecting something, a harness reaching
something. Prove it with a throwaway spike first, delete the spike, and write
only what it established. A mechanism assumed and written down costs an
amendment at best and a week of designing around a phantom constraint at worst.

**Have the LLD read adversarially before committing it.** `cargo lid-rs
lld-check` gates the mechanical properties; the `lid-rs-lld-review` agent reads
for what those cannot see — a section describing a mechanism a later amendment
replaced, a rule thin enough that a phase must guess, a gate obligation the
document never acknowledges. It reports what a phase would predictably stop on;
it cannot approve or block. Run it on the draft, and **again whenever
amendments have accumulated**: a document amended a dozen times is no longer
the document that was reviewed, and accretion is where inconsistency collects.

A claim added by an amendment after Phase 4 has run has no phase left that can
cite it — `src/lib.rs` and the hand-authored edges belong to Phases 3 and 4, so
Phase 7 fails check 10 on a file its hook forbids it to open. Add the edge with
the amendment, or reopen Phase 4.

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
