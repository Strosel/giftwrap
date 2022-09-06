use {
    crate::{get_field, GetFieldError},
    harled::FromDeriveInput,
    helpers::{generate_inner_conversions, get_wrap_depth, subtypes_list},
    proc_macro2::TokenStream,
    quote::{quote, quote_spanned},
    std::collections::HashSet,
    syn::{self, spanned::Spanned, GenericParam, Type},
};

#[macro_use]
mod helpers;

#[derive(FromDeriveInput, Debug)]
pub(crate) enum Derive {
    Struct(Struct),
    Enum(Enum),
}

impl Derive {
    pub(crate) fn derive(self) -> TokenStream {
        match self {
            Self::Struct(s) => s.derive(),
            Self::Enum(e) => e.derive(),
        }
    }
}

#[derive(FromDeriveInput, Debug)]
#[Struct]
pub(crate) struct Struct {
    ident: syn::Ident,
    generics: syn::Generics,
    fields: syn::Fields,
}

impl Struct {
    fn derive(self) -> TokenStream {
        let Self {
            ident,
            generics,
            fields,
        } = self;

        let mut stream = TokenStream::new();
        let generic_idents: HashSet<_> = generics
            .params
            .iter()
            .filter_map(|p| match p {
                GenericParam::Type(t) => Some(t.ident.to_string()),
                _ => None,
            })
            .collect();

        match get_field(&fields) {
            Err(GetFieldError::Unit) => cannot_wrap!(ident.span() => for "Unit struct"),
            Err(GetFieldError::NotSingle(span)) => cannot_wrap!(span => only "struct with 1 field"),
            Ok(field) => {
                let wrap_depth = match get_wrap_depth(&field.attrs) {
                    Ok(v) => v,
                    Err(e) => return e,
                };

                let types = subtypes_list(
                    &field.ty,
                    match wrap_depth {
                        0 => None,
                        n => Some(n),
                    },
                );

                if types.len() > 1
                    && types
                        .iter()
                        .filter_map(|ty| {
                            if let Type::Path(p) = ty {
                                Some(p.path.segments[0].ident.to_string())
                            } else {
                                None
                            }
                        })
                        .any(|ident| generic_idents.contains(&ident))
                {
                    return cannot_wrap!(fields.span() => "Generic type cannot be wrapped without causing conflicting implementations\n\tConsider using #[noWrap] or #[wrapDepth] here");
                }

                for (i, ty) in types.iter().enumerate() {
                    let froms = generate_inner_conversions(&types[..i]);
                    let from_ty = match &field.ident {
                        Some(ident) => quote! {
                            Self{ #ident: #froms }
                        },
                        None => quote! {
                            Self(#froms)
                        },
                    };
                    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
                    stream.extend::<TokenStream>( quote! {
                impl #impl_generics std::convert::From<#ty> for #ident #ty_generics #where_clause {
                    fn from(f: #ty) -> Self {
                        #from_ty
                    }
                }
            }
            );
                }
                stream
            }
        }
    }
}

#[derive(FromDeriveInput, Debug)]
#[Enum]
pub(crate) struct Enum {
    ident: syn::Ident,
    generics: syn::Generics,
    variants: Vec<syn::Variant>,
}

impl Enum {
    fn derive(self) -> TokenStream {
        let Self {
            ident,
            generics,
            variants,
        } = self;

        let mut wraps: HashSet<Type> = HashSet::new();
        let mut stream = TokenStream::new();

        let mut generic_wrap = false;
        let generic_idents: HashSet<_> = generics
            .params
            .iter()
            .filter_map(|p| match p {
                GenericParam::Type(t) => Some(t.ident.to_string()),
                _ => None,
            })
            .collect();

        for var in variants.iter() {
            let mut no_wrap = false;
            let wrap_depth = match get_wrap_depth(&var.attrs) {
                Ok(v) => v,
                Err(e) => return e,
            };

            for attr in var.attrs.iter() {
                if attr.path.is_ident("noWrap") {
                    no_wrap = true;
                }
            }

            if !no_wrap {
                match get_field(&var.fields) {
                    Err(GetFieldError::Unit) => {
                        return cannot_wrap!(var.span() => for "Unit variant");
                    }
                    Err(GetFieldError::NotSingle(span)) => {
                        return cannot_wrap!(span => only "variant with 1 field");
                    }
                    Ok(field) => {
                        let types = subtypes_list(
                            &field.ty,
                            match wrap_depth {
                                0 => None,
                                n => Some(n),
                            },
                        );

                        generic_wrap |= types
                            .iter()
                            .filter_map(|ty| {
                                if let Type::Path(p) = ty {
                                    Some(p.path.segments[0].ident.to_string())
                                } else {
                                    None
                                }
                            })
                            .any(|ident| generic_idents.contains(&ident));
                        if generic_wrap && !wraps.is_empty() {
                            return cannot_wrap!(var.fields.span() => "Generic type cannot be wrapped without causing conflicting implementations\n\tConsider using #[noWrap] or #[wrapDepth] here");
                        }

                        for (i, ty) in types.iter().enumerate() {
                            if wraps.insert(ty.clone()) {
                                let froms = generate_inner_conversions(&types[..i]);

                                let varname = &var.ident;
                                let from_ty = match &field.ident {
                                    Some(ident) => quote! {
                                        Self::#varname{ #ident: #froms }
                                    },
                                    None => quote! {
                                        Self::#varname(#froms)
                                    },
                                };
                                let (impl_generics, ty_generics, where_clause) =
                                    generics.split_for_impl();

                                stream.extend::<TokenStream>(
                            quote! {
                                impl #impl_generics std::convert::From<#ty> for #ident #ty_generics #where_clause {
                                    fn from(f: #ty) -> Self {
                                        #from_ty
                                    }
                                }
                            }
                        );
                            } else {
                                return cannot_wrap!(var.span() => "Cannot derive Wrap for two variants with the same inner type\n\tConsider using #[noWrap] or #[wrapDepth] here");
                            }
                        }
                    }
                }
            }
        }

        stream
    }
}
