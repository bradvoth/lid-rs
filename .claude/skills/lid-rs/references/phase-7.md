# Phase 7 — Gate, then commit

In order, all gating (README §4.5 verbatim — a change to either copy must
reach both):

```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --no-deps
cargo test --doc
cargo test --lib
cargo package -p <crate> --allow-dirty   # published crates: tarball builds standalone
cargo lid-rs sync --check  # the skill matches the lid-rs the project depends on
cargo lid-rs mutants       # diff scope; --full / --diff-base <ref> override
```

A failed gate never has "commit anyway" as an option: fix it, prove the tool
wrong with a reproducer, or stop (see `references/gates.md` for what each
check firing means). A gate that prints nothing is a result to report ("0
mutants in scope"), not silence.

Commit the slice as `phase 7: <version>: <what and why>` — the tag first,
then the project's normal convention — with the phase history legible in
the body. This commit covers Phase 6's implementation together with the
passing gate, since Phase 6 has no stop of its own, and the tag is what
makes the `commit-msg` hook run the full gate on it (the list above, as
`cargo lid-rs phase-check 7`) before it exists: a gate the agent ran by
hand is a gate the agent could have skipped. It is typically the branch's
last phase commit before opening the PR; the PR is then reviewable by
walking phases 1 through 7 in order.
