//! The rules are checked in the order check 13 states and the first failure is
//! reported, so a claim with several faults is fixed one at a time rather than
//! reported once ambiguously.
//!
//! This claim has three: it ends with no period, its verb is in no lexicon, and
//! it uses a term of the vague-term list. The terminator is the rule that
//! stands first, so the terminator is what the message names.

fn main() {}

/// When the [`Turn`] ends, the run shall frobnicate the [`Log`] quickly
#[derive(lid_rs::Spec)]
struct AClaimWithThreeFaults;
