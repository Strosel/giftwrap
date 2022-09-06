use {
    crate::GetFieldError,
    proc_macro2::Span,
    quote::quote,
    syn::{self, Attribute, GenericArgument, LitInt, PathArguments, Type},
};

pub(crate) enum Error {
    For(Span, &'static str),
    Only(Span, &'static str),
    Special(Span, &'static str),
}

impl From<Error> for syn::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::For(span, msg) => {
                syn::Error::new(span, format!("Wrap cannot be derived for {msg}"))
            }
            Error::Only(span, msg) => {
                syn::Error::new(span, format!("Wrap can only be derived for {msg}"))
            }
            Error::Special(span, msg) => syn::Error::new(span, msg),
        }
    }
}

impl From<GetFieldError> for Error {
    fn from(e: GetFieldError) -> Self {
        match e {
            GetFieldError::Unit(span) => Error::For(span, "Unit variant"),
            GetFieldError::NotSingle(span) => Error::Only(span, "variant with 1 field"),
        }
    }
}

pub(super) fn get_wrap_depth(attrs: &[Attribute]) -> Result<u32, Error> {
    if let Some(attr) = attrs.iter().find(|&a| (*a).path.is_ident("wrapDepth")) {
        match attr
            .parse_args::<LitInt>()
            .and_then(|l| l.base10_parse::<u32>())
        {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::Special(
                e.span(),
                "wrapDepth must be an unsigned integer",
            )),
        }
    } else {
        Ok(1)
    }
}

pub(super) fn generate_inner_conversions(types: &[Type]) -> proc_macro2::TokenStream {
    types
        .iter()
        .rev()
        .fold(quote! {f}, |froms, s_ty| match s_ty {
            Type::Path(path) => {
                let gen = path.path.segments[0].ident.clone();
                quote! {
                    #gen::<_>::from(#froms)
                }
            }
            Type::Ptr(_) | Type::Reference(_) => {
                quote! {&#froms}
            }
            _ => froms,
        })
}

pub(super) fn subtypes_list(top: &syn::Type, depth: Option<u32>) -> Vec<syn::Type> {
    let mut vec = vec![];

    let mut current = top;
    loop {
        if depth.map_or(false, |d| vec.len() == d as usize) {
            break;
        }
        vec.push(current.clone());
        match current {
            Type::Path(path) => {
                if let PathArguments::AngleBracketed(brac) = &path.path.segments[0].arguments {
                    if let Some(next_ty) = brac
                        .args
                        .iter()
                        .filter_map(|v| {
                            if let GenericArgument::Type(ty) = v {
                                Some(ty)
                            } else {
                                None
                            }
                        })
                        .next()
                    {
                        current = next_ty
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            Type::Ptr(ptr) => current = &*ptr.elem,
            Type::Reference(reference) => current = &*reference.elem,
            Type::Paren(p) => {
                vec.pop();
                current = &*p.elem
            }

            _ => break,
        }
    }

    vec
}
