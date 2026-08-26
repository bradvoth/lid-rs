{{#include ../../README.md:header}}

---

## About this book

Everything here is assembled by inclusion from the repository's living
artifacts — the specification (`README.md`), the workspace's own HLD and
LLDs under `docs/intent/`, and the operating skill. Nothing is written
twice: if this book and the repository ever disagree, one of them failed
its build.

- **The Specification** — the LID-rs methodology itself, section by
  section. Start at [The premise](spec/premise.md).
- **The Reference Implementation** — the design documents of the `lid-rs`,
  `lid-rs-macros`, and `xtask` crates, which were built by applying the
  methodology to itself. Their Decisions & Alternatives tables record
  what building it disproved and revised.
- **Operating It** — [the skill](skill.md) an agent follows to run the
  phase flow on a Rust codebase.

Read [LID](https://linked-intent.dev/) for the *why* of intent arrows;
read this book for what happens when the arrow is made of Rust items and
the compiler enforces every edge.
