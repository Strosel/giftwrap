use {
    proc_macro2::Span,
    serde::Deserialize,
    syn::{self, Attribute},
};

#[derive(Deserialize, Default, Debug)]
pub(crate) struct StructAttributes {
    #[serde(alias = "wrapDepth", default)]
    wrap_depth: Option<u32>,
}

impl StructAttributes {
    pub(crate) fn load(attrs: &[Attribute]) -> Result<Self, (Span, &'static str)> {
        attrs
            .iter()
            .find(|&a| (*a).path.is_ident("giftwrap"))
            .map(|attr| {
                let group: proc_macro2::Group = syn::parse(attr.tokens.clone().into())
                    .map_err(|e| (e.span(), "Attr is not a group"))?;
                if group.delimiter() == proc_macro2::Delimiter::Parenthesis {
                    let s = group.stream().to_string().replace(',', "\n");
                    Ok(toml::from_str(&s).map_err(|_| (group.span(), "Attr parse failed"))?)
                } else {
                    Err((group.span(), "Attr is not paren delimitered"))
                }
            })
            .unwrap_or_else(|| Ok(Self::default()))
    }

    pub(crate) fn wrap_depth(&self) -> Option<u32> {
        match self.wrap_depth {
            None | Some(0) => None,
            Some(n) => Some(n),
        }
    }
}

#[derive(Deserialize, Default, Debug)]
pub(crate) struct VariantAttributes {
    #[serde(alias = "wrapDepth", default)]
    wrap_depth: Option<u32>,
    #[serde(alias = "noWrap", default)]
    pub no_wrap: bool,
    #[serde(alias = "noUnwrap", default)]
    pub no_unwrap: bool,
}

impl VariantAttributes {
    pub(crate) fn load(attrs: &[Attribute]) -> Result<Self, (Span, &'static str)> {
        attrs
            .iter()
            .find(|&a| (*a).path.is_ident("giftwrap"))
            .map(|attr| {
                let group: proc_macro2::Group = syn::parse(attr.tokens.clone().into())
                    .map_err(|e| (e.span(), "Attr is not a group"))?;
                if group.delimiter() == proc_macro2::Delimiter::Parenthesis {
                    let s = group.stream().to_string().replace(',', "\n");
                    Ok(toml::from_str(&s).map_err(|_| (group.span(), "Attr parse failed"))?)
                } else {
                    Err((group.span(), "Attr is not paren delimitered"))
                }
            })
            .unwrap_or_else(|| Ok(Self::default()))
    }

    pub(crate) fn wrap_depth(&self) -> Option<u32> {
        match self.wrap_depth {
            None | Some(0) => None,
            Some(n) => Some(n),
        }
    }
}
