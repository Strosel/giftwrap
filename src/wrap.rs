use {
    proc_macro::TokenStream,
    quote::{quote, quote_spanned},
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

pub(crate) fn derive_wrap_struct(name: &syn::Ident, data: &syn::DataStruct) -> TokenStream {
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
        let from_ty = match &field.ident {
            Some(ident) => quote! {
                Self{ #ident: f }
            },
            None => quote! {
                Self(f)
            },
        };
        return quote! {
            impl From<#ty> for #name {
                fn from(f: #ty) -> Self {
                    #from_ty
                }
            }
        }
        .into();
    }
}
