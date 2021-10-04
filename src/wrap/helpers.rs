use {
    quote::{quote, quote_spanned},
    syn::{self, Attribute, GenericArgument, LitInt, PathArguments, Type},
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

pub(super) fn get_wrap_depth(attrs: &[Attribute]) -> Result<u32, proc_macro::TokenStream> {
    if let Some(attr) = attrs.iter().find(|&a| (*a).path.is_ident("wrapDepth")) {
        match attr
            .parse_args::<LitInt>()
            .and_then(|l| l.base10_parse::<u32>())
        {
            Ok(v) => Ok(v),
            Err(e) => Err(cannot_wrap! {
                e.span() => "wrapDepth must be an unsigned integer"
            }
            .into()),
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
