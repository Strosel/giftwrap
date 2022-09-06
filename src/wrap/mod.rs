use {
    crate::get_field,
    harled::FromDeriveInput,
    helpers::{generate_inner_conversions, get_wrap_depth, subtypes_list},
    proc_macro2::TokenStream,
    quote::quote,
    std::collections::HashSet,
    syn::{self, spanned::Spanned, GenericParam, Type},
};

mod helpers;
pub(crate) use helpers::Error;

#[derive(FromDeriveInput, Debug)]
pub(crate) enum Derive {
    Struct(Struct),
    Enum(Enum),
}

impl Derive {
    pub(crate) fn derive(self) -> TokenStream {
        let res = match self {
            Self::Struct(s) => s.derive(),
            Self::Enum(e) => e.derive(),
        };

        match res {
            Ok(derive) => derive,
            Err(e) => syn::Error::from(e).to_compile_error(),
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
    fn derive(self) -> Result<TokenStream, Error> {
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

        let field = get_field(&fields)?;
        let wrap_depth = get_wrap_depth(&field.attrs)?;
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
            return Err(Error::Special(fields.span(), "Generic type cannot be wrapped without causing conflicting implementations\n\tConsider using #[noWrap] or #[wrapDepth] here"));
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
            stream.extend::<TokenStream>(quote! {
                impl #impl_generics std::convert::From<#ty> for #ident #ty_generics #where_clause {
                    fn from(f: #ty) -> Self {
                        #from_ty
                    }
                }
            });
        }
        Ok(stream)
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
    fn derive(self) -> Result<TokenStream, Error> {
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
            let wrap_depth = get_wrap_depth(&var.attrs)?;
            for attr in var.attrs.iter() {
                if attr.path.is_ident("noWrap") {
                    no_wrap = true;
                }
            }

            if !no_wrap {
                let field = get_field(&var.fields)?;

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
                    return Err(Error::Special(var.fields.span() , "Generic type cannot be wrapped without causing conflicting implementations\n\tConsider using #[noWrap] or #[wrapDepth] here"));
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
                        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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
                        return Err(Error::Special(var.span() , "Cannot derive Wrap for two variants with the same inner type\n\tConsider using #[noWrap] or #[wrapDepth] here"));
                    }
                }
            }
        }

        Ok(stream)
    }
}
