extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use {
    syn,
    syn::{punctuated::Punctuated, spanned::Spanned, token},
};

#[macro_use]
mod wrap;
#[macro_use]
mod unwrap;

#[macro_export]
#[proc_macro_derive(Wrap)]
pub fn derive_wrap(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(s) => wrap::derive_wrap_struct(name, s),
        syn::Data::Enum(e) => cannot_wrap!(name.span() => for "enum (yet)").into(),
        syn::Data::Union(u) => cannot_wrap!(u.union_token.span => for "Union").into(),
    }
}

#[macro_export]
#[proc_macro_derive(Unwrap)]
pub fn derive_unwrap(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(s) => unwrap::derive_unwrap_struct(name, s),
        syn::Data::Enum(e) => cannot_unwrap!(name.span() => for "enum (yet)").into(),
        syn::Data::Union(u) => cannot_unwrap!(u.union_token.span => for "Union").into(),
    }
}
