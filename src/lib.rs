extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote_spanned;
use syn;

#[macro_use]
mod wrap;
#[macro_use]
mod unwrap;

/// Derve macro for `From<T>` where `T` is the inner type(s) of your struct or enum.
///
/// Any enum variant annotated with `#[noWrap]` will be ignored.
///
/// # Example
/// ```
/// #[derive(Wrap)]
/// enum SomeEnum {
///     Number(i64),
///     Text(String),
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
/// ```
#[proc_macro_derive(Wrap, attributes(noWrap))]
pub fn derive_wrap(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(s) => wrap::derive_wrap_struct(name, s),
        syn::Data::Enum(e) => wrap::derive_wrap_enum(name, e),
        syn::Data::Union(u) => cannot_wrap!(u.union_token.span => for "Union").into(),
    }
}

/// Derve macro for `impl From<S> for T` and `impl TryFrom<E> for T` for structs (`S`) and enums (`E`) where `T` is the inner type(s).
///
/// Any enum variant annotated with `#[noUnwrap]` will be ignored.
///
/// # Example
/// ```
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
///     fn from(f: SomeEnum) -> Self {
///         match f {
///             SomeEnum::Number(v) => v,
///             SomeEnum::Text(_) => "Cannot convert SomeEnum::Text into i64",
///             //...
///     }
/// }
///
/// impl TryFrom<SomeEnum> for String {
///     type Error = &'static str;
///
///     fn from(f: SomeEnum) -> Self {
///         match f {
///             SomeEnum::Text(v) => v,
///             SomeEnum::Number(_) => "Cannot convert SomeEnum::Number into String",
///             //...
///     }
/// }
/// ```
#[proc_macro_derive(Unwrap, attributes(noUnwrap))]
pub fn derive_unwrap(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(s) => unwrap::derive_unwrap_struct(name, s),
        syn::Data::Enum(e) => unwrap::derive_unwrap_enum(name, e),
        syn::Data::Union(u) => cannot_unwrap!(u.union_token.span => for "Union").into(),
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
