//! Which files of one crate the pass reads, and what it cannot read.
//!
//! The walk starts at the crate's root source files and follows the module
//! declarations it finds, so the files read are the ones a `mod` declaration
//! reaches and a declaration the pass cannot follow is a finding rather than
//! silence. Nothing here is resolved: a `#[cfg]` over a module declaration is
//! not evaluated, and a `#[path]` is read as the literal it holds.
//!
//! One crate, and one crate only. A path leading out of the crate the walk was
//! given is the open question this module carries — see [`declared_file`].

use std::path::{Path, PathBuf};

use lid_rs::implements;
use quote::ToTokens;

use crate::{Finding, spec};

/// One reading of a crate's source: the files it parsed, each with the tokens
/// that file holds, and the findings the reading raised on the way.
type Sources = (Vec<(PathBuf, syn::File)>, Vec<Finding>);

/// Every source file of one crate, parsed once, in the order the walk reached
/// them, beside the findings the reading raised.
pub(crate) fn sources(crate_root: &Path) -> Sources {
    read_all(&crate_roots(crate_root))
}

/// The crate's root source files — the `src/lib.rs` and `src/main.rs` it has —
/// and none at all when it has no `src` directory.
///
/// The empty answer is the whole of what a member with nothing to classify
/// gets: no file is read, so no shape is answered and no finding is raised.
#[implements(spec::ACrateWithNoSrcDirectoryIsTheEmptyClassification)]
fn crate_roots(crate_root: &Path) -> Vec<PathBuf> {
    ["lib.rs", "main.rs"].iter().map(|root| crate_root.join("src").join(root)).filter(|file| file.is_file()).collect()
}

/// Every file of `files`, and everything the module declarations those files
/// hold reach, read in the order they were named.
fn read_all(files: &[PathBuf]) -> Sources {
    files.iter().map(|file| read(file)).fold((Vec::new(), Vec::new()), merged)
}

/// One file: its tokens and everything its module declarations reach, or the
/// finding that stands in place of all of it.
fn read(file: &Path) -> Sources {
    match parse_source(file) {
        Ok(parsed) => read_parsed(file, parsed),
        Err(finding) => (Vec::new(), vec![finding]),
    }
}

/// One parsed file, then everything the module declarations it holds reach.
fn read_parsed(file: &Path, parsed: syn::File) -> Sources {
    let (declared, findings) = submodule_files(file, &parsed);
    merged((vec![(file.to_path_buf(), parsed)], findings), read_all(&declared))
}

/// Two readings in order: the first's files then the second's, and the first's
/// findings then the second's.
fn merged(first: Sources, second: Sources) -> Sources {
    (first.0.into_iter().chain(second.0).collect(), first.1.into_iter().chain(second.1).collect())
}

/// One source file's tokens, or the finding a file `syn` cannot parse raises.
///
/// The failure is answered, never raised: a file this pass cannot read is a
/// gap in the coverage of every rule at once, and the pass says so and reads
/// the rest of the crate.
#[implements(spec::AFileSynCannotParseIsAFindingAndNotAPanic)]
fn parse_source(file: &Path) -> Result<syn::File, Finding> {
    let unread = |saw: String| Finding::Unparsable { file: file.to_path_buf(), saw };
    let text = std::fs::read_to_string(file).map_err(|refusal| unread(refusal.to_string()))?;
    syn::parse_file(&text).map_err(|error| unread(error.to_string()))
}

/// The files the module declarations of one parsed file stand for, and the
/// findings for the declarations the pass cannot follow.
fn submodule_files(file: &Path, parsed: &syn::File) -> (Vec<PathBuf>, Vec<Finding>) {
    let dir = module_dir(file);
    let named: Vec<Result<PathBuf, Finding>> = declared_modules(parsed)
        .into_iter()
        .map(|(under, module)| declared_file(&dir.join(under), file, module))
        .collect();
    (named.iter().cloned().filter_map(Result::ok).collect(), named.into_iter().filter_map(Result::err).collect())
}

/// The directory the modules a file declares are looked for in: the file's own
/// directory for a crate root and a `mod.rs`, and the directory named for the
/// file otherwise.
fn module_dir(file: &Path) -> PathBuf {
    let directory = file.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = file.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    if ROOT_STEMS.contains(&stem.as_str()) { directory } else { directory.join(stem) }
}

/// The file names whose modules are looked for beside them rather than under a
/// directory of their own: a crate's two roots, and a directory's own file.
const ROOT_STEMS: [&str; 3] = ["lib", "main", "mod"];

/// Every module declaration one file's tokens make that stands for another
/// file, each with the directory its file is looked for in relative to the
/// directory of the file read — the empty path for a declaration at the file's
/// top level, and the module names above it for one nested in inline modules.
///
/// Every declaration, gated or not. A `#[cfg]` is an attribute the pass reads
/// and does not evaluate, so the file a gated declaration stands for is walked
/// like any other and the functions it holds are classified as written.
#[implements(spec::ACfgGatedModuleIsClassifiedAsWritten)]
fn declared_modules(parsed: &syn::File) -> Vec<(PathBuf, &syn::ItemMod)> {
    let mut standing = Vec::new();
    let mut pending: Vec<(PathBuf, &syn::Item)> =
        parsed.items.iter().rev().map(|item| (PathBuf::new(), item)).collect();
    while let Some((under, item)) = pending.pop() {
        let syn::Item::Mod(module) = item else { continue };
        match &module.content {
            Some((_, held)) => pending.extend(held.iter().rev().map(|item| (under.join(module.ident.to_string()), item))),
            None => standing.push((under, module)),
        }
    }
    standing
}

/// The file one module declaration stands for, under `dir`: the `#[path]` the
/// declaration carries when that attribute names a string literal, and the
/// `<module>.rs` or `<module>/mod.rs` beside it when it carries none — or the
/// finding, against `from`, that the declaration names a file the pass cannot
/// know.
///
/// **The bound on a literal `#[path]` is undecided and is not decided here.**
/// The design says a `#[path]` module is followed as a file path when the
/// attribute is a literal and states no bound on that path, while the same
/// document binds the pass to one crate's source. A literal naming a file
/// outside the crate the walk was given satisfies the first and breaks the
/// second, and a pair of literals naming each other has no end. This is the
/// item a bound would be written in; it holds none.
#[implements(spec::APathAttributeThatIsNoLiteralIsReportedUnreachable)]
fn declared_file(dir: &Path, from: &Path, module: &syn::ItemMod) -> Result<PathBuf, Finding> {
    let Some(attribute) = module.attrs.iter().find(|attribute| attribute.path().is_ident("path")) else {
        return Ok(beside(dir, &module.ident.to_string()));
    };
    if let syn::Meta::NameValue(named) = &attribute.meta
        && let syn::Expr::Lit(literal) = &named.value
        && let syn::Lit::Str(named_file) = &literal.lit
    {
        return Ok(dir.join(named_file.value()));
    }
    Err(Finding::Unreachable {
        file: from.to_path_buf(),
        module: module.ident.to_string(),
        saw: attribute.to_token_stream().to_string(),
    })
}

/// The file a module declaration carrying no `#[path]` stands for: the
/// `<module>/mod.rs` under `dir` where that is a file, and the `<module>.rs`
/// beside it otherwise.
///
/// The `.rs` is also the answer where neither is a file, so a declaration
/// standing for nothing is reported against the name a reader would look for.
fn beside(dir: &Path, module: &str) -> PathBuf {
    let nested = dir.join(module).join("mod.rs");
    if nested.is_file() { nested } else { dir.join(format!("{module}.rs")) }
}
