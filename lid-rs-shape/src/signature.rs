//! One function's written signature tokens, in both positions.

use lid_rs::implements;

use crate::{Signature, function::Function, spec};

/// One function's [`Signature`]: the file and name it is joined on, and the
/// type tokens of both positions.
pub(crate) fn signature_of(function: &Function) -> Signature {
    Signature {
        file: function.file.clone(),
        function: function.sig.ident.to_string(),
        parameters: parameter_tokens(&function.sig),
        returns: return_tokens(&function.sig),
    }
}

/// One entry per parameter, each the type tokens the source wrote for it, in
/// declaration order.
///
/// The tokens as written: what a `use` or a type alias would turn a name into
/// is a resolution, and this pass resolves nothing. A receiver is a parameter
/// like any other where the declaration wrote a type for it.
#[implements(spec::EveryFunctionOfTheCrateHasItsSignatureTokens)]
fn parameter_tokens(sig: &syn::Signature) -> Vec<String> {
    todo!("the type tokens of the {} parameters of `{}`", sig.inputs.len(), sig.ident)
}

/// The return type tokens the source wrote, whole, and none for a function
/// declared with no `->` at all.
///
/// Whole: a `Result<T, E>` is carried as it was written and not as the `T` dug
/// out of it. Rule V reads the `Ok` type from this and a consumer that wants
/// the whole type cannot recover it from a part.
#[implements(spec::TheReturnTokensAreTheWholeWrittenTypeAndNotTheOkTypeAlone)]
fn return_tokens(sig: &syn::Signature) -> Option<String> {
    todo!("the whole written return type of `{}`", sig.ident)
}

#[cfg(test)]
mod tests {
    //! The return position, over signatures parsed from source.
    //!
    //! The tokens are compared with their spacing removed. What a written type
    //! *is* belongs to this claim; how a printer spaces it does not, and a test
    //! that pinned the spacing would be a test about the printer.

    use lid_rs::validates;

    use super::*;

    /// The signature a fixture wrote, parsed.
    fn signature(source: &str) -> syn::Signature {
        syn::parse_str::<syn::ItemFn>(source).expect("a function the fixture wrote").sig
    }

    /// Type tokens without the spacing a printer chooses.
    fn tight(tokens: &str) -> String {
        tokens.replace(' ', "")
    }

    /// The return is carried as the whole written type — the `Result` and not
    /// the `Ok` type dug out of it — and as none where the declaration wrote no
    /// `->` at all.
    #[test]
    #[validates(spec::TheReturnTokensAreTheWholeWrittenTypeAndNotTheOkTypeAlone)]
    fn the_return_tokens_are_the_whole_written_type_and_not_the_ok_type_alone() {
        let whole = return_tokens(&signature("fn read(path: &Path) -> Result<Report, Error> { todo!() }"));
        let nothing = return_tokens(&signature("fn record(report: &Report) { todo!() }"));
        assert_eq!(
            (whole.as_deref().map(tight), nothing),
            (Some("Result<Report,Error>".to_string()), None),
            "the whole written return type, so a consumer wanting the `Ok` type can read it out and one wanting the whole has it",
        );
    }
}
