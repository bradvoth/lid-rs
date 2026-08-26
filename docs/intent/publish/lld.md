# Publishability — the crates build from their tarballs

## Context and Design Philosophy

A project adopts LID-rs by writing `lid-rs = "0.1"` in its `Cargo.toml`. That
line only works if the library crates exist on crates.io under names that are
free, and if the tarball `cargo package` produces compiles on its own —
without the workspace around it. Neither holds today, and both failures are
layout decisions the specification made for an *application* that turn out to
be wrong for a *published library workspace*:

- **The name `lid-rs` is taken.** An unrelated crate holds it on crates.io, and
  the citation macros hard-code `::lid_rs::` into every expansion. Every
  published artifact takes the project's own name as its prefix — `lid-rs`
  (Rust path `lid_rs`), `lid-rs-macros`, and the `cargo-lid-rs` subcommand
  to come — so that nothing of ours is ever read as a companion of the crate
  that holds `lid-rs`, and the expansions follow the runtime crate's new path.
  The rename is done as a rename — not as a `package = "lid-rs"` alias that
  every consumer would have to remember — because the alias moves the failure
  to the one place no gate reaches: a hand-written manifest.
- **`include_str!("../../docs/intent/…")` reaches outside the package root.**
  A tarball contains only files under the package directory, so every
  `#[doc = include_str!]` of a root-level intent document fails the publish
  verification build. Intent documents move under the crate that includes
  them. README §11's layout is not wrong for a single-package project, where
  `docs/intent/` already sits inside the package root; it is incomplete for a
  workspace, and the rule it was missing is: **an intent document lives inside
  the package root of the crate that includes it.**

The prefix rule reaches the tool namespace too: the metadata table is
`[workspace.metadata.lid_rs]` and macro diagnostics are prefixed `lid-rs:`.
`lid-rs` alone names the methodology in prose and nothing else.

Publishability is a gate, not a checklist item. `cargo package` runs the
verification build from the tarball; it joins §4.5 so that a future
`include_str!` pointing outside a package root fails the repository's own
gate, the way a moved book include already does.

## Layout after the change

```
Cargo.toml                       workspace; [workspace.package] readme/repository/homepage inherited
README.md                        the specification; readme.workspace = true copies it into each tarball
LICENSE                          Apache-2.0, workspace root; copied into each publishable crate
docs/intent/
  book/lld.md                    workspace-only artifacts: included by no crate, read by the book
  skill/lld.md
  publish/lld.md                 this document
lid-rs/
  Cargo.toml                     name = "lid-rs"; keywords, categories
  LICENSE
  docs/intent/hld.md             -> #![doc = include_str!("../docs/intent/hld.md")]
  docs/intent/registry/lld.md    -> pub mod registry
  docs/intent/intent-graph/lld.md-> pub mod graph
  src/…                          extern crate self as lid_rs
lid-rs-macros/
  LICENSE
  docs/intent/macros/lld.md      -> crate root doc
  src/expand.rs                  emits ::lid_rs::…
xtask/
  docs/intent/xtask/lld.md       -> crate root doc (unpublished, but the rule is uniform)
book/src/impl/*.md               {{#include}} paths follow the documents
```

The HLD is the workspace's design and is included as `lid-rs`'s crate-root
documentation, so it lives under `lid-rs`. The book and skill LLDs describe
artifacts no crate includes; they stay at the workspace root, which is the
precedent this document itself follows.

## Rename cascade

The rename is a Phase-8 cascade with the compiler naming the sites:
`extern crate self as lid_rs;` first, then every `::lid_rs::` token in
`lid-rs-macros/src/expand.rs` and in the hand-expanded canary and
`macro_edge!` registrations, then `cargo check` walks the `use lid_rs::` and
`lid_rs_macros::` sites. Four classes of site the compiler does *not* name, each
caught by a later gate step:

| Site | Caught by |
|---|---|
| `Spec::NAME` and edge-item strings in the pin tests (`module_path!()`-derived, `"lid_rs::spec::…"`) | `cargo test --lib`, at run time |
| The registry LLD's live doctest of the expansion contract | `cargo test --doc` |
| `trybuild` `.stderr` expectations echoing `#[lid_rs::implements]` and the `lid:` diagnostic prefix | `cargo test --lib` (regenerated, not hand-edited) |
| The dumped-registry assertion in `xtask`, the metadata pointer it reads, and the fixture manifest it synthesizes (`lid = { path = … }`) | `cargo xtask mutants`, `cargo xtask gate-selftest` |

No claim changes: the claims describe registration behaviour, and none names
the crate. A claim that expansions resolve through the published name would
restate "the macros compile" and duplicate the pin tests, which already assert
the exact `lid_rs::…` registry contents — the ceremonial kind Phase 2
rejects. The HLD's non-goal "no dependency-rename support" stands — the crate
is now depended on as `lid-rs`, and renaming *that* remains unsupported.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Runtime crate name | `lid-rs`, path `lid_rs`; directory renamed to match | `lid = { package = "lid-rs" }` alias with `::lid_rs::` expansions kept; a new name (`linked-intent`); no crates.io publish, git dependencies | The alias is zero code change but every consumer's manifest must carry it, and the failure mode (`unresolved crate lid` at expansion) lands on whoever forgot. The project is already called LID-rs. Git dependencies forgo semver and docs.rs. Directory follows package name so `path = "lid-rs"` and the package agree. |
| Companion names | `lid-rs-macros`, `cargo-lid-rs`, metadata key `lid_rs`, diagnostic prefix `lid-rs:` | `lid-rs-macros` + `cargo-lid` (free on crates.io, shorter to type) | On crates.io a `foo-macros` or `cargo-foo` name reads as belonging to `foo` — the `serde_derive`, `tokio-macros`, `cargo-pgrx` convention — and `foo` here is someone else's crate. One rule, no exceptions, is cheaper to state than a list of which names are ours; the typing cost of `cargo lid-rs` is paid by us, the confusion cost by strangers. |
| Where intent docs live | Under the including crate's package root, at `<crate>/docs/intent/…`; workspace-only LLDs stay at `docs/intent/` | Symlinks from each crate into `docs/intent/`; keep root docs and drop the `include_str!` from published crates; a build script copying docs in | Symlinks make the checkout depend on git `core.symlinks` and Windows developer mode. Dropping the include severs the LLD from `cargo doc` — the reason it is included at all. A build script is a dependency to solve a `mv`. The uniform rule costs one path segment in the book. |
| Crate-root documentation of `lid-rs` | The workspace HLD, as today | The README (the specification) | `readme.workspace = true` already ships the specification as the crates.io landing page; the crate-root doc is where the reference implementation's design belongs. Revisit if docs.rs readers ask for the spec first. |
| License | Apache-2.0 only; `LICENSE` at the workspace root and copied into each publishable crate | Dual `MIT OR Apache-2.0` (the ecosystem convention); `license-file` pointing at the root | The owner's choice of terms. Verified: with an SPDX `license`, `cargo package` copies a workspace `readme` into a member's tarball but not root license files, and `license-file` is mutually exclusive with `license`. Copies are what `serde_derive` ships. |
| Gate addition | `cargo package --workspace` (verification build from tarballs) after `cargo test --lib`, in all four copies of the gate list (README §4.5, the skill, `CLAUDE.md`, `gate.yml`) | Publish-time only; `--no-verify` | A gate that exists, gates: the verification build is the only thing that proves the tarball compiles, and a root-relative `include_str!` is exactly the drift it catches. The four copies already disagreed (`mdbook build book` in two of them); this slice reconciles them by stating the rule in §4.5: the spec's list is the floor, and a workspace appends its own build-integrity steps after it. |
| Publish metadata | `repository`, `readme`, `homepage` (the book), `license` inherited from `[workspace.package]`; `keywords`, `categories` per crate | Per-crate duplication | One canonical home per setting, as §7 prescribes for lints. |

## Open Questions & Future Decisions

### Deferred
1. `cargo publish --workspace` itself — a manual step until a release
   workflow exists; the gate proves publishability, not publication.
2. Whether `lid-rs-macros` should gain a `readme` of its own rather than
   shipping the specification twice.

## References

- [§7 Configuration](https://bradvoth.github.io/lid-rs/spec/configuration.html) — one canonical home per setting.
- [§11 Repo layout](https://bradvoth.github.io/lid-rs/spec/layout.html) — revised by this slice to state the package-root rule.
- [§4.5 The gate](https://bradvoth.github.io/lid-rs/spec/gates.html) — where `cargo package` joins.
- [The Cargo Book: `cargo package`](https://doc.rust-lang.org/cargo/commands/cargo-package.html) — tarball contents are the package root only.
