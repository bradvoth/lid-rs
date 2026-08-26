# High-Level Design: __LID_PACKAGE_NAME__

## Problem

What is wrong, for whom, and why it matters. Written so a reader with no
access to this conversation understands the need.

## Approach

The shape of the solution in a few paragraphs: the mechanisms that carry it
and why they were chosen over the alternatives. Rationale that cannot be
recovered from code belongs here.

## Target Users

Who operates the system and what they need from it.

## Goals

Falsifiable, in delivery order. Each goal names the observable that proves it.

1. …

## Non-Goals

What this system deliberately does not do, so the omission is read as a
decision rather than an oversight.

## Tenets

Ordered; the earlier tenet wins on conflict.

1. …

## System Design

Components and the edges between them. A diagram is welcome where it shows a
mechanism prose would obscure.

### Slice map (delivery order)

| # | Slice (user-visible operation) | Delivers |
|---|---|---|
| 1 | "…" | … |

Each slice runs Phases 0–7 of the LID-rs flow with a stop at every phase
boundary; its LLD lives at `docs/intent/<slice>/lld.md` and is included as the
documentation of the module that implements it.

## Key Design Decisions

| Decision | Alternatives considered | Rationale |
|---|---|---|
| … | … | … |

## Success Metrics

How the system is judged working, and the signals that would falsify it.

## References

- [LID-rs](https://bradvoth.github.io/lid-rs/) — the methodology and its
  specification.
