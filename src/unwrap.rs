use {
    crate::{get_field, GetFieldError},
    proc_macro::TokenStream,
    quote::{quote, quote_spanned},
    std::collections::{HashMap, HashSet},
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

pub(crate) fn derive_unwrap_struct(
    name: &syn::Ident,
    data: &syn::DataStruct,
    generics: syn::Generics,
) -> TokenStream {
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
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        (quote! {
            impl #impl_generics std::convert::From<#name #ty_generics> for #ty #where_clause {
                fn from(f: #name #ty_generics) -> Self {
                    #from_self
                }
            }
        })
        .into()
    }
}

struct Variant<'a> {
    var: &'a syn::Variant,
    field: &'a syn::Field,
}

pub(crate) fn derive_unwrap_enum(
    name: &syn::Ident,
    data: &syn::DataEnum,
    generics: syn::Generics,
) -> TokenStream {
    let mut wraps: HashMap<&syn::Type, Vec<Variant>> = HashMap::new();
    let mut all_variants: HashSet<&syn::Variant> = HashSet::new();
    let mut stream = TokenStream::new();

    for var in data.variants.iter() {
        all_variants.insert(var);
        let mut no_unwrap = false;

        for attr in var.attrs.iter() {
            if attr.path.is_ident("noUnwrap") {
                no_unwrap = true;
            }
        }

        if !no_unwrap {
            match get_field(&var.fields) {
                Err(GetFieldError::Unit) => {
                    return cannot_unwrap!(var.span() => for "Unit variant").into();
                }
                Err(GetFieldError::NotSingle(span)) => {
                    return cannot_unwrap!(span => only "variant with 1 field").into();
                }
                Ok(field) => {
                    let ty: &syn::Type = &field.ty;
                    match wraps.get_mut(ty) {
                        Some(v) => {
                            v.push(Variant { var, field });
                        }
                        None => {
                            wraps.insert(ty, vec![Variant { var, field }]);
                        }
                    }
                }
            }
        }
    }
    for (ty, vars) in wraps.iter() {
        let mut match_arms = Vec::new();
        for var in vars.iter() {
            let varname = &var.var.ident;
            match_arms.push(match &var.field.ident {
                Some(ident) => quote! {
                    #name::#varname{ #ident } => Ok(#ident),
                },
                None => quote! {
                    #name::#varname(v) => Ok(v),
                },
            })
        }
        let err_arms: Vec<_> = all_variants.difference(&vars.iter().map(|var| var.var).collect()).map(|var| {
            let ident = &var.ident;
            match var.fields {
                syn::Fields::Named(_) => quote! {#name::#ident{..} => Err(concat!("Can't convert ", stringify!(#name), "::", stringify!(#ident), " into ", stringify!(#ty))),},
                syn::Fields::Unnamed(_) => quote! {#name::#ident(..) => Err(concat!("Can't convert ", stringify!(#name), "::", stringify!(#ident), " into ", stringify!(#ty))),},
                syn::Fields::Unit => quote! {#name::#ident => Err(concat!("Can't convert ", stringify!(#name), "::", stringify!(#ident), " into ", stringify!(#ty))),},
            }
        }).collect();
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        stream.extend::<TokenStream>(
            quote! {
                impl #impl_generics  std::convert::TryFrom<#name #ty_generics> for #ty #where_clause {
                    type Error = &'static str;

                    fn try_from(f: #name #ty_generics) -> std::result::Result<Self, Self::Error> {
                        match f {
                            #(#match_arms)*
                            #(#err_arms)*
                        }
                    }
                }
            }
            .into(),
        );
    }
    stream
}
