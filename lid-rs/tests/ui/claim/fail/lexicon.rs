//! The verb and the vague-term rules, against the base lexicon alone.
//!
//! trybuild compiles every fixture as a bin of one package it generates, whose
//! parent directory holds no manifest, so the walk for a project lexicon stops
//! at once and the base is the whole lexicon a fixture answers to. `frobnicate`
//! is therefore in no lexicon here, and `quickly` is in the built-in
//! vague-term list, which no project file can take from.
//!
//! A term of that list is matched as a whole word, or as a phrase when it holds
//! a space, and without regard to case — so a capitalised occurrence and a
//! two-word one fail as the plain one does. The message names the term as the
//! list writes it, not as the claim spells it.

fn main() {}

/// When the [`Turn`] ends, the run shall frobnicate the [`Log`].
#[derive(lid_rs::Spec)]
struct AVerbTheBaseDoesNotDefine;

/// When the [`Turn`] ends, the run shall stop quickly.
#[derive(lid_rs::Spec)]
struct ABuiltInVagueTerm;

/// When the [`Turn`] ends, the run shall stop Quickly.
#[derive(lid_rs::Spec)]
struct ACapitalisedVagueTerm;

/// When the [`Turn`] ends, the run shall stop as needed.
#[derive(lid_rs::Spec)]
struct AMultiWordVagueTerm;
