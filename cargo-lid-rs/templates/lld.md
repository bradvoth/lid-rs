# __LID_PACKAGE_NAME__ — the crate root as the first slice

## Context and Design Philosophy

A freshly initialised package has no slice directory, so its first slice is
the crate root: this document is the slice's design, `src/spec.rs` beside it
is the slice's claims file, and `src/lib.rs` includes both. The package keeps
this shape until a user-visible operation earns its own `src/<slice>/`
directory with an `lld.md` of its own; from then on this document describes
only what remains at the root.

## Behaviour

State here what the crate root does for its user: the operation, its inputs,
its outcome, and the errors it can name. Every claim in `src/spec.rs` must be
derivable from this section.

## Decisions & Alternatives

| Decision | Chosen | Alternatives considered | Rationale |
|---|---|---|---|
| Where the first slice lives | The crate root, with `src/lld.md` and `src/spec.rs` | A named `src/<slice>/` directory from the first commit | The package has no operation to name yet; a directory named by guess would be renamed by the first real slice |
