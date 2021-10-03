use {
    crate::{get_field, subtypes_list, GetFieldError},
    helpers::get_wrap_depth,
    proc_macro::TokenStream,
    quote::{quote, quote_spanned},
    std::collections::HashSet,
    syn,
    syn::{spanned::Spanned, GenericParam, LitInt, Type},
};

#[macro_use]
mod helpers;

pub(crate) fn derive_wrap_struct(
    name: &syn::Ident,
    data: &syn::DataStruct,
    generics: syn::Generics,
) -> TokenStream {
    let mut stream = TokenStream::new();
    let generic_idents: HashSet<_> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(t) => Some(t.ident.to_string()),
            _ => None,
        })
        .collect();

    match get_field(&data.fields) {
        Err(GetFieldError::Unit) => cannot_wrap!(name.span() => for "Unit struct").into(),
        Err(GetFieldError::NotSingle(span)) => {
            cannot_wrap!(span => only "struct with 1 field").into()
        }
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
                return cannot_wrap!(data.fields.span() => "Generic type cannot be wrapped without causing conflicting implementations\n\tConsider using #[noWrap] or #[wrapDepth] here").into();
            }

            for (i, ty) in types.iter().enumerate() {
                let mut froms = quote! {f};
                if i != 0 {
                    froms = types[..i]
                        .iter()
                        .rev()
                        .filter_map(|s_ty| {
                            if let Type::Path(p) = s_ty {
                                Some(p.path.segments[0].ident.clone())
                            } else {
                                None
                            }
                        })
                        .fold(froms, |froms, gen| {
                            quote! {
                                #gen::<_>::from(#froms)
                            }
                        })
                }
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
                impl #impl_generics std::convert::From<#ty> for #name #ty_generics #where_clause {
                    fn from(f: #ty) -> Self {
                        #from_ty
                    }
                }
            }
            .into());
            }
            stream
        }
    }
}

pub(crate) fn derive_wrap_enum(
    name: &syn::Ident,
    data: &syn::DataEnum,
    generics: syn::Generics,
) -> TokenStream {
    let mut wraps: HashSet<syn::Type> = HashSet::new();
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

    for var in data.variants.iter() {
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
                    return cannot_wrap!(var.span() => for "Unit variant").into();
                }
                Err(GetFieldError::NotSingle(span)) => {
                    return cannot_wrap!(span => only "variant with 1 field").into();
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
                    if generic_wrap && wraps.len() != 0 {
                        return cannot_wrap!(var.fields.span() => "Generic type cannot be wrapped without causing conflicting implementations\n\tConsider using #[noWrap] or #[wrapDepth] here").into();
                    }

                    for (i, ty) in types.iter().enumerate() {
                        if wraps.insert(ty.clone()) {
                            let mut froms = quote! {f};
                            if i != 0 {
                                froms = types[..i]
                                    .iter()
                                    .rev()
                                    .filter_map(|s_ty| {
                                        if let Type::Path(p) = s_ty {
                                            Some(p.path.segments[0].ident.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .fold(froms, |froms, gen| {
                                        quote! {
                                            #gen::<_>::from(#froms)
                                        }
                                    })
                            }

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
                                impl #impl_generics std::convert::From<#ty> for #name #ty_generics #where_clause {
                                    fn from(f: #ty) -> Self {
                                        #from_ty
                                    }
                                }
                            }
                            .into(),
                        );
                        } else {
                            return cannot_wrap!(var.span() => "Cannot derive Wrap for two variants with the same inner type\n\tConsider using #[noWrap] or #[wrapDepth] here").into();
                        }
                    }
                }
            }
        }
    }

    stream
}
