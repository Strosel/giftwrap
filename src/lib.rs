extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote_spanned;
use syn;

#[macro_use]
mod wrap;
#[macro_use]
mod unwrap;

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
