//! The functions one parsed file holds, as tokens.
//!
//! Which functions a crate has is one decision and it is made here, once, for
//! both of the crate's answers: a shape is answered for each of these and a
//! signature is carried for each of these, so the two join on a file and a
//! name because they were read from the same list.

use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::spec;

/// One function's tokens, as one file wrote them.
///
/// The pieces are the written ones and nothing is derived here: whether the
/// declaration says `pub`, whether it carries a mark, and what its body is
/// shaped like are each read from these by the item that answers for them.
pub(crate) struct Function {
    /// The file the tokens were read from.
    pub(crate) file: PathBuf,
    /// The visibility the declaration wrote.
    pub(crate) vis: syn::Visibility,
    /// The attributes the declaration carries, a mark among them or not.
    pub(crate) attrs: Vec<syn::Attribute>,
    /// The signature the declaration wrote, its name and its type tokens among
    /// it.
    pub(crate) sig: syn::Signature,
    /// The body the declaration wrote, which is what F1 through F6 read.
    pub(crate) block: syn::Block,
}

/// Every function one file's tokens hold, in the order the file wrote them:
/// the free functions, the methods of its `impl` blocks, and the trait methods
/// it gives a body to, at any depth of inline module.
///
/// Every function, whether the verdict for it is flow or leaf, because the
/// signature tokens a conformance check reads are wanted for all of them and a
/// verdict is wanted for all of them.
///
/// Inline modules are descended into whether the source gates them or not. A
/// `#[cfg]` is an attribute the pass reads and does not evaluate, so the
/// functions a gated module holds are classified as written; a function a
/// macro would generate is not here to be read at all, its invocation in the
/// enclosing body being the whole of what the pass can see of it.
#[implements(
    spec::EveryFunctionOfTheCrateHasItsSignatureTokens,
    spec::ACfgGatedModuleIsClassifiedAsWritten,
)]
pub(crate) fn functions_in(file: &Path, parsed: &syn::File) -> Vec<Function> {
    functions_among(file, &parsed.items)
}

/// Every function the items hold, in the order they were written.
fn functions_among(file: &Path, items: &[syn::Item]) -> Vec<Function> {
    items.iter().flat_map(|item| functions_of(file, item)).collect()
}

/// Every function one item holds: the function the item is, the methods an
/// `impl` block holds, the methods a trait declaration gives a body to, and
/// everything an inline module holds.
///
/// An item of any other kind holds none. A trait's method declared without a
/// body is a signature and not a function, so nothing is answered for it; a
/// trait's method has no visibility of its own — it is as public as its trait
/// — so the declaration is read as writing none.
fn functions_of(file: &Path, item: &syn::Item) -> Vec<Function> {
    if let syn::Item::Fn(declared) = item {
        return vec![function(file, &declared.vis, &declared.attrs, &declared.sig, &declared.block)];
    }
    if let syn::Item::Impl(block) = item {
        return block
            .items
            .iter()
            .filter_map(|member| {
                let syn::ImplItem::Fn(method) = member else { return None };
                Some(function(file, &method.vis, &method.attrs, &method.sig, &method.block))
            })
            .collect();
    }
    if let syn::Item::Trait(declared) = item {
        return declared
            .items
            .iter()
            .filter_map(|member| {
                let syn::TraitItem::Fn(method) = member else { return None };
                let body = method.default.as_ref()?;
                Some(function(file, &syn::Visibility::Inherited, &method.attrs, &method.sig, body))
            })
            .collect();
    }
    if let syn::Item::Mod(module) = item {
        return module.content.iter().flat_map(|(_, items)| functions_among(file, items)).collect();
    }
    Vec::new()
}

/// One function as the pass carries one, from the pieces a declaration wrote.
fn function(
    file: &Path,
    vis: &syn::Visibility,
    attrs: &[syn::Attribute],
    sig: &syn::Signature,
    block: &syn::Block,
) -> Function {
    Function {
        file: file.to_path_buf(),
        vis: vis.clone(),
        attrs: attrs.to_vec(),
        sig: sig.clone(),
        block: block.clone(),
    }
}
