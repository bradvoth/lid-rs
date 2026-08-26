# The book — the methodology, published

## Context and Design Philosophy

The specification, the reference implementation's design documents, and the
operating skill are the project's public face; a rustdoc-styled mdBook on
GitHub Pages makes them readable without cloning the repo. Tenet 1 makes the
README the single living statement of the methodology; applied to this book,
that means **no prose of its own beyond the landing page.** Every chapter is
assembled by `{{#include}}`
from the living artifacts — `README.md` sliced by invisible `<!-- ANCHOR -->`
comments into one chapter per section, the HLD and LLDs included whole, the
skill included past its frontmatter. A forked copy would drift; an include
cannot.

The landing page states the lineage explicitly: LID-rs is an opinionated,
Rust-specific implementation of Linked-Intent Development
(<https://linked-intent.dev/>), which is the source of the idea; the same
attribution leads the README.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Generator | mdBook, default (rustdoc-style) theme | Static site generators; hand-rolled HTML; docs.rs only | "Rustdoc-style book" is mdBook's native look; docs.rs already renders the crate docs but not the spec or skill. No theme work wanted or needed. |
| Content sourcing | `{{#include}}` with `ANCHOR` comments in `README.md` / `SKILL.md`; whole-file includes for HLD and LLDs | Splitting `README.md` into per-chapter files; copying prose into `book/src` | Splitting demotes the README from canonical artifact to build product; copying is drift by construction. Anchor comments are invisible in GitHub rendering and stripped from includes. |
| Drift gating | `mdbook build book` runs in the main gate *and* gates the Pages deploy | Deploy-workflow-only build | A renamed anchor or moved doc must fail the repository's own gate (tenet 2), not surface later as a broken site. |
| Playground buttons | Disabled (`runnable = false`) | Default runnable rust blocks | The examples cite `lid-rs`, which the playground cannot resolve; a run button that always errors is noise. |
| Diagrams | Mermaid blocks render as code | `mdbook-mermaid` preprocessor | One fewer binary in two workflows; the HLD's diagram reads acceptably as text. Revisit if diagrams multiply (tenet 3 escalation path). |
| §-references | Markdown links to the published book page, by absolute URL, in every markdown artifact | Same-page `#anchor` links; book-relative paths; a link-rewriting preprocessor | The artifacts render in three contexts: GitHub, this book's sliced chapters, and rustdoc. Only an absolute URL resolves identically in all three — page anchors break once the README is sliced, and relative paths break outside the book. A preprocessor could rewrite links per context, but the absolute form makes that tooling unnecessary (tenet 3). Page-level targets, not heading fragments — fragment slugs are a rendering detail no gate checks. |

## Open Questions & Future Decisions

### Deferred
1. `mdbook-mermaid` once rendered diagrams earn the extra install.
2. A custom domain, if the project outgrows `bradvoth.github.io/lid-rs`.

## References

- <https://linked-intent.dev/> — LID, the source methodology.
- [mdBook documentation](https://rust-lang.github.io/mdBook/) — include and
  anchor semantics this build relies on.
