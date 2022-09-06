use {
    crate::{get_field, GetFieldError},
    harled::FromDeriveInput,
    proc_macro2::{Span, TokenStream},
    quote::quote,
    std::collections::{HashMap, HashSet},
    syn::{punctuated::Punctuated, token},
};

pub(crate) enum Error {
    For(Span, &'static str),
    Only(Span, &'static str),
}

impl From<Error> for syn::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::For(span, msg) => {
                syn::Error::new(span, format!("Unwrap cannot be derived for {msg}"))
            }
            Error::Only(span, msg) => {
                syn::Error::new(span, format!("Unwrap can only be derived for {msg}"))
            }
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

        let (fields, err_span): (&Punctuated<syn::Field, token::Comma>, proc_macro2::Span) =
            match &fields {
                syn::Fields::Named(f) => (&f.named, f.brace_token.span),
                syn::Fields::Unnamed(f) => (&f.unnamed, f.paren_token.span),
                syn::Fields::Unit => {
                    return Err(Error::For(ident.span(), "Unit struct"));
                }
            };

        if fields.len() != 1 {
            return Err(Error::Only(err_span, "struct with 1 field"));
        }

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
        Ok(quote! {
            impl #impl_generics std::convert::From<#ident #ty_generics> for #ty #where_clause {
                fn from(f: #ident #ty_generics) -> Self {
                    #from_self
                }
            }
        })
    }
}

#[derive(FromDeriveInput, Debug)]
#[Enum]
pub(crate) struct Enum {
    ident: syn::Ident,
    generics: syn::Generics,
    variants: Vec<syn::Variant>,
}

struct Variant<'a> {
    var: &'a syn::Variant,
    field: &'a syn::Field,
}

impl Enum {
    fn derive(self) -> Result<TokenStream, Error> {
        let Self {
            ident: name,
            generics,
            variants,
        } = self;

        let mut wraps: HashMap<&syn::Type, Vec<Variant>> = HashMap::new();
        let mut all_variants: HashSet<&syn::Variant> = HashSet::new();
        let mut stream = TokenStream::new();

        for var in variants.iter() {
            all_variants.insert(var);
            let mut no_unwrap = false;

            for attr in var.attrs.iter() {
                if attr.path.is_ident("noUnwrap") {
                    no_unwrap = true;
                }
            }

            if !no_unwrap {
                let field = get_field(&var.fields)?;

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
        );
        }
        Ok(stream)
    }
}
