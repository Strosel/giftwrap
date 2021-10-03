use {
    proc_macro::TokenStream,
    quote::quote_spanned,
    syn::{Attribute, LitInt},
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

pub(super) fn get_wrap_depth(attrs: &Vec<Attribute>) -> Result<u32, TokenStream> {
    if let Some(attr) = attrs.iter().find(|&a| (*a).path.is_ident("wrapDepth")) {
        match attr
            .parse_args::<LitInt>()
            .and_then(|l| l.base10_parse::<u32>())
        {
            Ok(v) => Ok(v),
            Err(e) => Err(quote_spanned! {
                e.span() => compile_error!("wrapDepth must be an unsigned integer" );
            }
            .into()),
        }
    } else {
        Ok(1)
    }
}
