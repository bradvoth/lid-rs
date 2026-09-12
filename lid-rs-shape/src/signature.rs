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
