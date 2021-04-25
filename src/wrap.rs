use {
    crate::{get_field, GetFieldError},
    proc_macro::TokenStream,
    quote::{quote, quote_spanned},
    std::collections::HashSet,
    syn,
    syn::spanned::Spanned,
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
    ($span:expr => $msg:expr) => {
        quote_spanned! {
            $span => compile_error!($msg);
        }
    };
}

pub(crate) fn derive_wrap_struct(name: &syn::Ident, data: &syn::DataStruct) -> TokenStream {
    match get_field(&data.fields) {
        Err(GetFieldError::Unit) => cannot_wrap!(name.span() => for "Unit struct").into(),
        Err(GetFieldError::NotSingle(span)) => {
            cannot_wrap!(span => only "struct with 1 field").into()
        }
        Ok(field) => {
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
                impl std::convert::From<#ty> for #name {
                    fn from(f: #ty) -> Self {
                        #from_ty
                    }
                }
            }
            .into();
        }
    }
}

pub(crate) fn derive_wrap_enum(name: &syn::Ident, data: &syn::DataEnum) -> TokenStream {
    let mut wraps: HashSet<&syn::Type> = HashSet::new();
    let mut stream = TokenStream::new();

    for var in data.variants.iter() {
        let mut no_wrap = false;

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
                    let ty: &syn::Type = &field.ty;
                    if wraps.insert(ty) {
                        let varname = &var.ident;
                        let from_ty = match &field.ident {
                            Some(ident) => quote! {
                                Self::#varname{ #ident: f }
                            },
                            None => quote! {
                                Self::#varname(f)
                            },
                        };
                        stream.extend::<TokenStream>(
                            quote! {
                                impl std::convert::From<#ty> for #name {
                                    fn from(f: #ty) -> Self {
                                        #from_ty
                                    }
                                }
                            }
                            .into(),
                        );
                    } else {
                        return cannot_wrap!(var.span() => "Cannot derive Wrap for two variants with the same inner type\n\tConsider using #[noWrap]").into();
                    }
                }
            }
        }
    }

    stream
}
