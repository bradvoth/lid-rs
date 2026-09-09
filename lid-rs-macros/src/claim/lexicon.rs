//! The lexicon the derive executes: the verbs a claim's modal may take, each
//! with the signature templates conformance will one day match, and the terms
//! a claim may not use.
//!
//! The file gives every verb a definition too, and a verb without one — or with
//! two — fails every derive. The definition is for the reader of the file: no
//! rule asks it a question and no registration records it, so it is required
//! and not carried.
//!
//! Two files make it. The base is compiled into this crate at its own build,
//! so a claim in a published crate — built by its consumers with nothing else —
//! always answers to the lexicon it was written against. A project's file,
//! found by walking up from `CARGO_MANIFEST_DIR`, adds to it.
//!
//! [`read`] is the whole of that composition, and [`Lexicon`]'s queries are
//! how the rules of the language ask it anything: a verb by name, the terms
//! the project prohibits beyond the built-in list, and the project file the
//! messages name.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The admitted verbs and the project's additions to the prohibited terms: the
/// base merged with the project's file, as one claim is checked against it.
///
/// The built-in vague-term list is a constant of the derive rather than a field
/// here, because every lexicon prohibits it; `extra` is only what a project's
/// file adds. The fields are private and read through the queries below, so
/// that a merged lexicon is asked rather than searched: `claim::parse` lives in
/// the parent module and every rule it applies is one of these three questions.
pub struct Lexicon {
    /// Each admitted verb, lower-cased, with the templates the file that
    /// supplied it gives it.
    verbs: BTreeMap<String, Verb>,
    /// The terms the project's `[prohibited] extra` adds, kept apart from the
    /// built-in list so that a match can name the file that supplied it.
    extra: Vec<String>,
    /// The project file the lexicon was extended by, absent when the walk found
    /// none: the path a message names, and the one the expansion `include_str!`s.
    project: Option<PathBuf>,
}

impl Lexicon {
    /// The verb this lexicon admits under `name`, or `None`.
    ///
    /// The one lookup the language turns on: a miss is the undefined-verb
    /// rule, and a hit carries whether the verb's meaning is a shape or a
    /// behaviour — so whether the claim needs a response-object link — and the
    /// templates the claim's parts record.
    pub fn verb(&self, name: &str) -> Option<&Verb> {
        let _ = name;
        todo!()
    }

    /// The terms the project's file prohibits beyond the built-in list.
    ///
    /// Kept apart from that list because a match here names the lexicon file
    /// that supplied the term as well as the term, and a match on the built-in
    /// list names only the term.
    pub fn extra(&self) -> &[String] {
        todo!()
    }

    /// The project file this lexicon was read from, absent when the walk found
    /// none.
    ///
    /// The file an undefined verb's message names beside the base, the file a
    /// project `extra` match names, and the file the expansion `include_str!`s
    /// so that editing it rebuilds every claim.
    pub fn project(&self) -> Option<&Path> {
        todo!()
    }
}

/// One admitted verb: the shape the signature of an implementer of a claim
/// using it takes, and so whether the claim needs a response-object link.
///
/// The verb's table holds a definition too, required and checked by [`parse`],
/// but a definition answers no question the rules ask: no message quotes it and
/// no registration records it. The glossary is the file; what is carried here
/// is what a rule reads.
pub struct Verb {
    /// The entries of the verb's `signature`, in the file's order.
    pub templates: Vec<Template>,
}

impl Verb {
    /// Whether a claim taking this verb needs a response-object link: a verb
    /// whose signature is templates rather than `*` names a return shape, and
    /// a shape has an object.
    pub fn is_shape(&self) -> bool {
        todo!()
    }
}

/// One entry of a verb's `signature`.
pub enum Template {
    /// `*`: the verb's meaning is behaviour rather than a return shape, so the
    /// claim needs no response-object link and the parts record `["*"]`.
    Any,
    /// A template as written — `->` followed by type tokens in which
    /// `{object}` and `{owner}` are placeholders and `_` stands for any single
    /// type. Recorded on the claim unmatched: matching it against a signature
    /// is conformance's.
    Shape(String),
}

impl Template {
    /// One `signature` entry as the file writes it: [`Template::Any`] for `*`,
    /// [`Template::Shape`] for a template whose syntax holds — it begins with
    /// `->`, its braces are balanced, and every placeholder is `{object}` or
    /// `{owner}`.
    ///
    /// The error names `file`, `verb`, and the entry, which is why the check
    /// is here rather than in the enum's shape: a template is the only part of
    /// the format whose failure needs the verb that carried it.
    pub fn parse(entry: &str, verb: &str, file: &Path) -> Result<Self, String> {
        let _ = (entry, verb, file);
        todo!()
    }
}

/// The base lexicon's file, named as a message names it.
///
/// Every crate answers to this file, so it is the half of "the files the
/// lexicon was read from" that no query returns: an undefined verb's message
/// names it and, when [`Lexicon::project`] has one, that file beside it.
pub const BASE_FILE: &str = "lid-rs-macros/lexicon.toml";

/// The lexicon a crate built at `manifest_dir` answers to: the base compiled
/// into this crate, extended by the project's file when [`locate`] finds one
/// and [`parse`] admits it.
///
/// A verb the project's file names is admitted with the templates that file
/// gives it, whether or not the base names the verb too: the project's entry
/// stands whole rather than merging key by key, so a redefined verb has one
/// meaning — the one its own file's `def` states — and not two halves of two
/// files' tables. Where the walk finds no file the base is the whole lexicon —
/// the case a consumer of a published crate builds in, and the reason a
/// published claim's verbs are the base's.
///
/// The base arrives as an `include_str!` of `lid-rs-macros/lexicon.toml`,
/// which the hand commit adds: until that file exists there is nothing to
/// include, so the inclusion lands with it.
pub fn read(manifest_dir: &Path) -> Result<Lexicon, String> {
    let _ = manifest_dir;
    todo!()
}

/// The project's lexicon file for a crate built at `manifest_dir`: the first
/// `docs/intent/lexicon.toml` in that directory or an ancestor.
///
/// The walk stops — without a file — when the directory it would move to holds
/// no `Cargo.toml`, and after examining a directory whose `Cargo.toml` has a
/// line that is exactly `[workspace]`. So a member finds its workspace's file,
/// and a crate built from the registry cache finds none.
pub fn locate(manifest_dir: &Path) -> Option<PathBuf> {
    let _ = manifest_dir;
    todo!()
}

/// The TOML subset to a lexicon, or the message naming the file and what in it
/// lies outside the subset — the line, the verb and the key, or the verb and
/// the template.
///
/// `def` and `signature` are each required exactly once under a verb, and a
/// verb missing or repeating either fails here naming the verb and the key.
/// Only `signature` reaches the [`Lexicon`]: the definition is read to be
/// required, not to be carried, so that the file stays a glossary a reader can
/// trust while the lexicon holds only what a rule asks.
///
/// The failure fails every claim in the crate under compilation, which is loud
/// by design: every claim depends on the file.
pub fn parse(text: &str, file: &Path) -> Result<Lexicon, String> {
    let _ = (text, file);
    todo!()
}
