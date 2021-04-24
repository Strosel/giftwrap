extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use {
    syn,
    syn::{punctuated::Punctuated, spanned::Spanned, token},
};

macro_rules! cannot_wrap {
    ($span:expr => for $name:expr) => {
        quote_spanned! {
            $span => compile_error!(concat!("Wrap cannot be derived for ", $name));
        }
    };
    ($span:expr => only $name:expr) => {
        quote_spanned! {
            $span => compile_error!(concat!("Wrap can only be derived for ", $name));
        }
    };
}

#[proc_macro_derive(Wrap)]
pub fn derive_wrap(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    match &ast.data {
        syn::Data::Struct(s) => derive_wrap_struct(name, s),
        syn::Data::Enum(e) => cannot_wrap!(name.span() => for "enum (yet)").into(),
        syn::Data::Union(u) => cannot_wrap!(u.union_token.span => for "Union").into(),
    }
}

fn derive_wrap_struct(name: &syn::Ident, data: &syn::DataStruct) -> TokenStream {
    let (fields, err_span): (&Punctuated<syn::Field, token::Comma>, proc_macro2::Span) =
        match &data.fields {
            syn::Fields::Named(f) => (&f.named, f.brace_token.span),
            syn::Fields::Unnamed(f) => (&f.unnamed, f.paren_token.span),
            syn::Fields::Unit => {
                return cannot_wrap!(name.span() => for "Unit struct").into();
            }
        };
    if fields.len() != 1 {
        cannot_wrap!(err_span => only "struct with 1 field").into()
    } else {
        let field: &syn::Field = fields.first().unwrap();
        let ty: &syn::Type = &field.ty;
        let (ty_self, self_ty) = match &field.ident {
            Some(ident) => (
                quote! {
                    Self{ #ident: f }
                },
                quote! {
                    f.#ident
                },
            ),
            None => (
                quote! {
                    Self(f)
                },
                quote! {
                    f.0
                },
            ),
        };
        return quote! {
            impl From<#ty> for #name {
                fn from(f: #ty) -> Self {
                    #ty_self
                }
            }
            impl From<#name> for #ty {
                fn from(f: #name) -> Self {
                    #self_ty
                }
            }
        }
        .into();
    }
}
