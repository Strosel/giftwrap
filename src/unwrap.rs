use {
    proc_macro::TokenStream,
    quote::{quote, quote_spanned},
    syn,
    syn::{punctuated::Punctuated, spanned::Spanned, token},
};

macro_rules! cannot_unwrap {
    ($span:expr => for $name:expr) => {
        quote_spanned! {
            $span => compile_error!(concat!("Unwrap cannot be derived for ", $name));
        }
    };
    ($span:expr => only $name:expr) => {
        quote_spanned! {
            $span => compile_error!(concat!("Unwrap can only be derived for ", $name));
        }
    };
}

pub(crate) fn derive_unwrap_struct(name: &syn::Ident, data: &syn::DataStruct) -> TokenStream {
    let (fields, err_span): (&Punctuated<syn::Field, token::Comma>, proc_macro2::Span) =
        match &data.fields {
            syn::Fields::Named(f) => (&f.named, f.brace_token.span),
            syn::Fields::Unnamed(f) => (&f.unnamed, f.paren_token.span),
            syn::Fields::Unit => {
                return cannot_unwrap!(name.span() => for "Unit struct").into();
            }
        };
    if fields.len() != 1 {
        cannot_unwrap!(err_span => only "struct with 1 field").into()
    } else {
        let field: &syn::Field = fields.first().unwrap();
        let ty: &syn::Type = &field.ty;
        let from_self = match &field.ident {
            Some(ident) => quote! {
                f.#ident
            },
            None => quote! {
                f.0
            },
        };
        return quote! {
            impl From<#name> for #ty {
                fn from(f: #name) -> Self {
                    #from_self
                }
            }
        }
        .into();
    }
}
