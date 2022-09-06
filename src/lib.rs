extern crate proc_macro;
use harled::{Error, Kind};
use proc_macro::TokenStream;
use quote::quote_spanned;

#[macro_use]
mod wrap;
#[macro_use]
mod unwrap;

/// Derve macro for `From<T>` where `T` is the inner type(s) of your struct or enum.
///
/// Using `#[wrapDepth(n)]` `From` is derived for every type in the chain, which is useful for
/// types such as `Box<T>` and `Arc<Mutex<T>>`. Setting wrapDepth to 0 will derive for all inner
/// types. Default depth is 1.
///
/// Any enum variant annotated with `#[noWrap]` will be ignored.
///
/// # Example
/// ```ignore
/// use std::sync::{Arc, Mutex};
/// use giftwrap::Wrap;
///
/// #[derive(Wrap)]
/// enum SomeEnum {
///     Number(i64),
///     Text(String),
///     #[wrapDepth(0)]
///     DeepVariant(Arc<Mutex<bool>>),
///     #[noWrap]
///     Real(f64),
/// }
///
/// //would generate
/// impl From<i64> for SomeEnum {
///     fn from(f: i64) -> Self {
///         SomeEnum::Number(f)
///     }
/// }
///
/// impl From<String> for SomeEnum {
///     fn from(f: String) -> Self {
///         SomeEnum::Text(f)
///     }
/// }
///
/// impl From<Arc<Mutex<bool>>> for SomeEnum {
///     fn from(f: Arc<Mutex<bool>>) -> Self {
///         SomeEnum::DeepVariant(f)
///     }
/// }
///
/// impl From<Mutex<bool>> for SomeEnum {
///     fn from(f: Mutex<bool>) -> Self {
///         SomeEnum::DeepVariant(Arc::<_>::from(f))
///     }
/// }
///
/// impl From<bool> for SomeEnum {
///     fn from(f: bool) -> Self {
///         SomeEnum::DeepVariant(Arc::<_>::from(Mutex::<_>::from(f)))
///     }
/// }
/// ```
#[proc_macro_derive(Wrap, attributes(noWrap, wrapDepth))]
pub fn derive_wrap(input: TokenStream) -> TokenStream {
    let wrap: Result<wrap::Derive, _> = harled::parse(input);
    match wrap {
        Ok(wrap) => wrap.derive().into(),
        Err(Error::Unsupported(Kind::Union, span)) => cannot_wrap!(span => for "Union").into(),
        Err(Error::Syn(syn)) => syn.to_compile_error().into(),
        _ => unreachable!(),
    }
}

/// Derve macro for `impl From<S> for T` and `impl TryFrom<E> for T` for structs (`S`) and enums (`E`) where `T` is the inner type(s).
///
/// Any enum variant annotated with `#[noUnwrap]` will be ignored.
///
/// # Example
/// ```ignore
/// use std::convert::TryFrom;
/// use giftwrap::Unwrap;
///
/// #[derive(Unwrap)]
/// enum SomeEnum {
///     Number(i64),
///     Text(String),
///     #[noUnwrap]
///     Real(f64),
/// }
///
/// //would generate
/// impl TryFrom<SomeEnum> for i64 {
///     type Error = &'static str;
///
///     fn try_from(f: SomeEnum) -> Self {
///         match f {
///             SomeEnum::Number(v) => v,
///             SomeEnum::Text(_) => "Cannot convert SomeEnum::Text into i64",
///             //...
///         }
///     }
/// }
///
/// impl TryFrom<SomeEnum> for String {
///     type Error = &'static str;
///
///     fn try_from(f: SomeEnum) -> Self {
///         match f {
///             SomeEnum::Text(v) => v,
///             SomeEnum::Number(_) => "Cannot convert SomeEnum::Number into String",
///             //...
///         }
///     }
/// }
/// ```
#[proc_macro_derive(Unwrap, attributes(noUnwrap))]
pub fn derive_unwrap(input: TokenStream) -> TokenStream {
    let unwrap: Result<unwrap::Derive, _> = harled::parse(input);
    match unwrap {
        Ok(unwrap) => unwrap.derive().into(),
        Err(Error::Unsupported(Kind::Union, span)) => cannot_unwrap!(span => for "Union").into(),
        Err(Error::Syn(syn)) => syn.to_compile_error().into(),
        _ => unreachable!(),
    }
}

pub(crate) enum GetFieldError {
    Unit,
    NotSingle(proc_macro2::Span),
}

pub(crate) fn get_field(fields: &syn::Fields) -> Result<&syn::Field, GetFieldError> {
    match fields {
        syn::Fields::Named(f) => {
            if f.named.len() != 1 {
                Err(GetFieldError::NotSingle(f.brace_token.span))
            } else {
                Ok(f.named.first().unwrap())
            }
        }
        syn::Fields::Unnamed(f) => {
            if f.unnamed.len() != 1 {
                Err(GetFieldError::NotSingle(f.paren_token.span))
            } else {
                Ok(f.unnamed.first().unwrap())
            }
        }
        syn::Fields::Unit => Err(GetFieldError::Unit),
    }
}
