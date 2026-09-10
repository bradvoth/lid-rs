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
//!
//! Below [`read`] the module falls into three: the walk that finds a file
//! ([`locate`]), the parser that turns one file's text into a [`Lexicon`]
//! ([`parse`], over the [`Line`] each line is classified as), and the template
//! syntax check ([`Template::parse`]). Each names the file in its failures,
//! because a lexicon failure fails every claim in the crate and the reader
//! needs to know which file did it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The base lexicon's text, compiled into this crate at its own build.
///
/// A published crate is built by its consumers with this and nothing else, so
/// the verbs a published claim uses are the verbs in here; the file is
/// versioned with the crates, and so a published claim always compiles under
/// the lexicon it was written against.
const BASE: &str = include_str!("../../lexicon.toml");

/// The `signature` entry that marks a verb's meaning as behaviour rather than
/// shape: the one string of the format that stands for something instead of
/// naming it.
const ANY: &str = "*";

/// The placeholders a template may hold, each binding to a part of the claim
/// that used the verb.
const PLACEHOLDERS: [&str; 2] = ["object", "owner"];

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
        self.verbs.get(name)
    }

    /// The terms the project's file prohibits beyond the built-in list.
    ///
    /// Kept apart from that list because a match here names the lexicon file
    /// that supplied the term as well as the term, and a match on the built-in
    /// list names only the term.
    pub fn extra(&self) -> &[String] {
        &self.extra
    }

    /// The project file this lexicon was read from, absent when the walk found
    /// none.
    ///
    /// The file an undefined verb's message names beside the base, the file a
    /// project `extra` match names, and the file the expansion `include_str!`s
    /// so that editing it rebuilds every claim.
    pub fn project(&self) -> Option<&Path> {
        self.project.as_deref()
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
        self.templates.iter().any(|template| matches!(template, Template::Shape(_)))
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
        if entry == ANY {
            return Ok(Self::Any);
        }
        if well_formed(entry) {
            return Ok(Self::Shape(entry.to_string()));
        }
        Err(malformed(entry, verb, file))
    }

    /// The entry as the file wrote it: `*` for a behaviour verb, the template
    /// itself for a shape.
    ///
    /// This is what a claim's parts record — the derive carries the templates
    /// unmatched, so the string a later check reads is the string a reader of
    /// the lexicon wrote.
    pub fn as_written(&self) -> &str {
        match self {
            Self::Any => ANY,
            Self::Shape(entry) => entry,
        }
    }
}

/// Whether a `signature` entry is a template the format admits: it begins with
/// `->`, its braces are balanced, and every name between them is one of
/// [`PLACEHOLDERS`].
fn well_formed(entry: &str) -> bool {
    entry.starts_with("->") && braces(entry).is_some_and(|names| names.iter().all(|n| PLACEHOLDERS.contains(n)))
}

/// The name inside each `{…}` of a template, in the order written; `None` when
/// the braces are unbalanced — a `{` no `}` closes, a `}` that closes nothing,
/// or a `{` inside a `{…}`.
fn braces(entry: &str) -> Option<Vec<&str>> {
    let mut spans = entry.split('{');
    spans.next().filter(|opening| !opening.contains('}'))?;
    spans
        .map(|span| span.split_once('}').and_then(|(name, rest)| (!rest.contains('}')).then_some(name)))
        .collect()
}

/// The template failure: the file, the verb that carried the entry, and the
/// entry itself — the row of the lexicon's failure table a template takes.
fn malformed(entry: &str, verb: &str, file: &Path) -> String {
    format!(
        "lid-rs: {}: a template begins with `->` and names only `{{object}}` and `{{owner}}`, \
         and the verb `{verb}` gives: {entry}",
        file.display()
    )
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
pub fn read(manifest_dir: &Path) -> Result<Lexicon, String> {
    let base = parse(BASE, Path::new(BASE_FILE))?;
    let Some(file) = locate(manifest_dir) else {
        return Ok(base);
    };
    let project = parse(&contents(&file)?, &file)?;
    Ok(merged(base, project, file))
}

/// A lexicon file's text, or the message naming the file and why it could not
/// be read.
///
/// The walk found this file, so failing to read it is the file's own failure
/// and not the absence [`locate`] reports by returning `None`.
fn contents(file: &Path) -> Result<String, String> {
    std::fs::read_to_string(file).map_err(|why| format!("lid-rs: {}: the lexicon cannot be read: {why}", file.display()))
}

/// The project's lexicon over the base: every verb the project names is
/// admitted whole — replacing the base's entry rather than merging with it —
/// the project's `extra` becomes the terms beyond the built-in list, and
/// `file` becomes the path [`Lexicon::project`] returns.
fn merged(base: Lexicon, project: Lexicon, file: PathBuf) -> Lexicon {
    let mut verbs = base.verbs;
    verbs.extend(project.verbs);
    Lexicon { verbs, extra: project.extra, project: Some(file) }
}

/// The project's lexicon file for a crate built at `manifest_dir`: the first
/// `docs/intent/lexicon.toml` in that directory or an ancestor.
///
/// The walk stops — without a file — when the directory it would move to holds
/// no `Cargo.toml`, and after examining a directory whose `Cargo.toml` has a
/// line that is exactly `[workspace]`. So a member finds its workspace's file,
/// and a crate built from the registry cache finds none.
pub fn locate(manifest_dir: &Path) -> Option<PathBuf> {
    walk(manifest_dir).into_iter().find_map(|dir| lexicon_in(&dir))
}

/// The directories the walk examines, nearest first: `manifest_dir` and the
/// ancestors it is allowed to move to.
///
/// The walk is bounded by the path's own ancestors, so it ends at the
/// filesystem root however the manifests read; it ends earlier at a directory
/// whose `Cargo.toml` has a line that is exactly `[workspace]`, which is
/// examined and then stops the walk, and before a directory holding no
/// `Cargo.toml`, which is never examined.
fn walk(manifest_dir: &Path) -> Vec<PathBuf> {
    manifest_dir
        .ancestors()
        .enumerate()
        .take_while(|(above, dir)| *above == 0 || has_manifest(dir))
        .scan(false, |stopped, (_, dir)| {
            let reached = *stopped;
            *stopped = workspace_root(dir);
            (!reached).then(|| dir.to_path_buf())
        })
        .collect()
}

/// The `docs/intent/lexicon.toml` under a directory, when the file is there.
fn lexicon_in(dir: &Path) -> Option<PathBuf> {
    let file = dir.join("docs/intent/lexicon.toml");
    file.is_file().then_some(file)
}

/// Whether a directory holds a `Cargo.toml`.
///
/// The walk may not move to one that does not, which is how a crate built from
/// the registry cache — whose parent directories are the cache's, not a
/// project's — finds no project lexicon and answers to the base alone.
fn has_manifest(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file()
}

/// Whether a directory's `Cargo.toml` has a line that is exactly
/// `[workspace]`.
///
/// Such a directory is the top of the project the crate belongs to: the walk
/// examines it and then stops, so a member finds its workspace's lexicon and
/// looks no further up.
fn workspace_root(dir: &Path) -> bool {
    std::fs::read_to_string(dir.join("Cargo.toml")).is_ok_and(|text| text.lines().any(|line| line == "[workspace]"))
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
/// The lexicon this returns names no project file: one file's text says
/// nothing about which of the two files it is, and [`read`] is what knows.
///
/// The failure fails every claim in the crate under compilation, which is loud
/// by design: every claim depends on the file.
pub fn parse(text: &str, file: &Path) -> Result<Lexicon, String> {
    let lines = classify(text, file)?;
    orphans(text, &lines, file)?;
    Ok(Lexicon {
        verbs: verbs(&lines, file)?,
        extra: extra(&lines, file)?,
        project: None,
    })
}

/// One line of the file, classified as one of the subset's forms.
///
/// The parse reads the file twice through this enum — once for the verb
/// tables, once for the prohibited table — so the classification is done once
/// and the forms are a closed set: a line that is none of them never becomes a
/// `Line` at all, because [`line`] rejects it naming the file and the line.
enum Line {
    /// A comment or a blank line, which the format ignores.
    Ignored,
    /// `[verbs.<name>]`, opening the verb it names.
    Verb(String),
    /// `[prohibited]`, opening the prohibited table.
    Prohibited,
    /// A `key = value` line: the key, and the strings its value holds — one for
    /// a quoted string, each element for a list.
    Entry(String, Vec<String>),
}

/// A table's entries: each key with the strings its value holds, in the file's
/// order, so a key given twice is two entries and the repetition is visible to
/// the rule that rejects it.
type Entries = Vec<(String, Vec<String>)>;

/// Every line of the file, classified, in order; the first line outside the
/// subset is the failure.
fn classify(text: &str, file: &Path) -> Result<Vec<Line>, String> {
    text.lines().map(|raw| line(raw, file)).collect()
}

/// One line as the form it takes, or the message naming the file and the line.
fn line(raw: &str, file: &Path) -> Result<Line, String> {
    let text = raw.trim();
    ignored(text)
        .or_else(|| header(text))
        .or_else(|| entry(text))
        .ok_or_else(|| broken_line(raw, file))
}

/// [`Line::Ignored`] for a comment — `#` to the end of the line — or a blank
/// line; `None` for anything else.
fn ignored(text: &str) -> Option<Line> {
    (text.is_empty() || text.starts_with('#')).then_some(Line::Ignored)
}

/// The table a header opens: [`Line::Verb`] for `[verbs.<name>]`, `<name>`
/// being lower-case letters, and [`Line::Prohibited`] for `[prohibited]`.
/// `None` when the line opens no table.
fn header(text: &str) -> Option<Line> {
    let named = text.strip_prefix('[')?.strip_suffix(']')?;
    match named.strip_prefix("verbs.") {
        Some(verb) => verb.chars().all(|c| c.is_ascii_lowercase()).then(|| Line::Verb(verb.to_string())),
        None => (named == "prohibited").then_some(Line::Prohibited),
    }
}

/// A `key = value` line: the key, and the strings its value holds. `None` when
/// the line is not a key and a value the subset admits.
fn entry(text: &str) -> Option<Line> {
    let (key, value) = text.split_once('=')?;
    Some(Line::Entry(key.trim().to_string(), values(value.trim())?))
}

/// The strings a value holds: one for a double-quoted string, each element for
/// a bracketed list of them. `None` when the text is neither, which is what
/// makes the line one the subset does not admit.
fn values(text: &str) -> Option<Vec<String>> {
    quoted(text).map(|s| vec![s.to_string()]).or_else(|| list(text))
}

/// The strings of a bracketed list: comma-separated, each double-quoted.
/// `None` when the text is not a list, or an element is not a string the
/// subset admits.
fn list(text: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = text.strip_prefix('[')?.strip_suffix(']')?.split('"').collect();
    let gaps: Vec<&str> = parts.iter().copied().step_by(2).collect();
    let last = gaps.len() - 1;
    let separated = parts.len() % 2 == 1
        && gaps.iter().enumerate().all(|(nth, gap)| gap.trim() == if nth == 0 || nth == last { "" } else { "," });
    let elements: Vec<String> = parts.iter().skip(1).step_by(2).map(|element| (*element).to_string()).collect();
    (separated && elements.iter().all(|element| !element.contains('\\'))).then_some(elements)
}

/// The contents of a double-quoted string holding no `"` and no backslash;
/// `None` for anything else.
fn quoted(text: &str) -> Option<&str> {
    text.strip_prefix('"')?.strip_suffix('"').filter(|inside| !inside.contains(['"', '\\']))
}

/// The line failure: the file and the line, which is the row of the lexicon's
/// failure table a line outside the subset takes.
fn broken_line(raw: &str, file: &Path) -> String {
    format!(
        "lid-rs: {}: a line of the lexicon is a comment, a table header, or a key and a value, and this one is none: {raw}",
        file.display()
    )
}

/// No entry stands before the file's first table header.
///
/// A key that opens no table belongs to no table, so where it stands it is a
/// line the subset does not admit — and it fails as one, by [`broken_line`],
/// naming the file and the line as written. The stray entry is the line rule's
/// case and not a rule of its own: the format's failures are the rows of the
/// LLD's table, and a key outside every table adds no row to it.
fn orphans(text: &str, lines: &[Line], file: &Path) -> Result<(), String> {
    match stray(text, lines) {
        Some(raw) => Err(broken_line(raw, file)),
        None => Ok(()),
    }
}

/// The first `key = value` line that no table header precedes, as the file
/// wrote it.
///
/// [`classify`] takes the file's lines in order and yields one [`Line`] each,
/// so the raw lines and their classifications run in step and the line the
/// message names is the text the author reads in the file.
fn stray<'a>(text: &'a str, lines: &[Line]) -> Option<&'a str> {
    text.lines()
        .zip(lines)
        .take_while(|(_, line)| !opens_table(line))
        .find(|(_, line)| is_entry(line))
        .map(|(raw, _)| raw)
}

/// Whether a line opens a table, which is where the search for a stray entry
/// ends: every entry after the first header belongs to the table above it.
fn opens_table(line: &Line) -> bool {
    matches!(line, Line::Verb(_) | Line::Prohibited)
}

/// Whether a line is a `key = value` entry — the one form that can stand
/// outside a table, and so the only one a stray can be.
fn is_entry(line: &Line) -> bool {
    matches!(line, Line::Entry(..))
}

/// The verb a header opens, when the line is one: the name `[verbs.<name>]`
/// carries.
///
/// The one reading of [`Line::Verb`], so that the tables the parse walks and
/// the names their failures report come from the same place.
fn verb_named(line: &Line) -> Option<&str> {
    match line {
        Line::Verb(name) => Some(name),
        Line::Ignored | Line::Prohibited | Line::Entry(..) => None,
    }
}

/// The key and the values a `key = value` line carries; `None` for any other
/// form.
///
/// The one reading of [`Line::Entry`], shared by the verb tables and the
/// prohibited table, because an entry means the same thing under either header.
fn pair(line: &Line) -> Option<(String, Vec<String>)> {
    match line {
        Line::Entry(key, values) => Some((key.clone(), values.clone())),
        Line::Ignored | Line::Verb(_) | Line::Prohibited => None,
    }
}

/// The file's verbs: each named at most once, each with `def` and `signature`
/// given exactly once, and each carrying only the templates a rule reads.
fn verbs(lines: &[Line], file: &Path) -> Result<BTreeMap<String, Verb>, String> {
    let tables = verb_tables(lines);
    named_once(&tables, file)?;
    tables
        .iter()
        .map(|(name, entries)| Ok((name.clone(), verb(name, entries, file)?)))
        .collect()
}

/// Each verb table in the file: the verb its header names, and the entries
/// between that header and the next, in the file's order.
fn verb_tables(lines: &[Line]) -> Vec<(String, Entries)> {
    lines
        .iter()
        .filter(|line| opens_table(line))
        .zip(lines.split(opens_table).skip(1))
        .filter_map(|(header, body)| verb_named(header).map(|name| (name.to_string(), entries_of(body))))
        .collect()
}

/// The entries of one table: the `key = value` lines between its header and the
/// next, in the file's order.
///
/// [`classify`] yields one [`Line`] per line of the file, so splitting that
/// sequence at the headers gives each table's body, and a key given twice is
/// two entries here — which is what lets the rule that rejects the repetition
/// see it.
fn entries_of(body: &[Line]) -> Entries {
    body.iter().filter_map(pair).collect()
}

/// Every verb is named once; the error names the file and the verb named
/// twice.
fn named_once(tables: &[(String, Entries)], file: &Path) -> Result<(), String> {
    let repeated = tables
        .iter()
        .enumerate()
        .find(|(at, (name, _))| tables[..*at].iter().any(|(earlier, _)| earlier == name));
    match repeated {
        Some((_, (name, _))) => {
            Err(format!("lid-rs: {}: the verb `{name}` opens more than one table", file.display()))
        }
        None => Ok(()),
    }
}

/// One verb's table: its keys are the two the format admits, each given
/// exactly once, and its `signature` becomes the templates the verb carries.
///
/// `def` is read here to be required and is then dropped, which is the whole
/// of "the file is the glossary": the reader gets a definition, the derive
/// gets what a rule asks for.
fn verb(name: &str, entries: &Entries, file: &Path) -> Result<Verb, String> {
    known_keys(entries, &["def", "signature"], file)?;
    once(entries, "def", name, file)?;
    let signature = once(entries, "signature", name, file)?;
    Ok(Verb {
        templates: templates(signature, name, file)?,
    })
}

/// Every key of a table is one the table admits; the error names the file and
/// the first key that is not.
fn known_keys(entries: &Entries, admitted: &[&str], file: &Path) -> Result<(), String> {
    match entries.iter().find(|(key, _)| !admitted.contains(&key.as_str())) {
        Some((key, _)) => Err(format!("lid-rs: {}: `{key}` is no key this table admits", file.display())),
        None => Ok(()),
    }
}

/// The values of a key given exactly once; the error names the file, the verb,
/// and the key that is missing or repeated.
fn once<'a>(entries: &'a Entries, key: &str, verb: &str, file: &Path) -> Result<&'a [String], String> {
    let given: Vec<&[String]> = entries.iter().filter(|(k, _)| k == key).map(|(_, values)| values.as_slice()).collect();
    match given[..] {
        [only] => Ok(only),
        _ => Err(format!(
            "lid-rs: {}: a verb gives `{key}` exactly once, and `{verb}` gives it {} times",
            file.display(),
            given.len()
        )),
    }
}

/// The values of a key a table may omit: absent, they are none; repeated, the
/// error names the file and the key.
fn at_most_once(entries: &Entries, key: &str, file: &Path) -> Result<Vec<String>, String> {
    let given: Vec<&Vec<String>> = entries.iter().filter(|(k, _)| k == key).map(|(_, values)| values).collect();
    match given[..] {
        [] => Ok(Vec::new()),
        [only] => Ok(only.clone()),
        _ => Err(format!(
            "lid-rs: {}: a table gives `{key}` once at most, and this one gives it {} times",
            file.display(),
            given.len()
        )),
    }
}

/// A verb's `signature` entries as templates, in the file's order.
fn templates(signature: &[String], verb: &str, file: &Path) -> Result<Vec<Template>, String> {
    signature.iter().map(|entry| Template::parse(entry, verb, file)).collect()
}

/// The terms the file's `[prohibited]` table adds to the built-in list; none
/// when the file opens no such table.
fn extra(lines: &[Line], file: &Path) -> Result<Vec<String>, String> {
    let entries = prohibited_entries(lines);
    known_keys(&entries, &["extra"], file)?;
    at_most_once(&entries, "extra", file)
}

/// The entries under every `[prohibited]` header, in the file's order.
fn prohibited_entries(lines: &[Line]) -> Entries {
    lines
        .iter()
        .filter(|line| opens_table(line))
        .zip(lines.split(opens_table).skip(1))
        .filter(|(header, _)| matches!(header, Line::Prohibited))
        .flat_map(|(_, body)| entries_of(body))
        .collect()
}
