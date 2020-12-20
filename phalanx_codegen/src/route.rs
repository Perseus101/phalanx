use std::convert::TryFrom;

// use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};

use syn::{parse::Parse, Attribute, FnArg, ImplItemMethod, LitStr, Pat, Path, Type};

use quote::{quote, ToTokens};

use route_attr::RouteAttr;

pub struct Route<'a> {
    method: &'a ImplItemMethod,
    server_type: &'a Type,
    args: Vec<&'a FnArg>,
    attrs: Vec<&'a Attribute>,
    route_attr: RouteAttr,
}

impl<'a> Route<'a> {
    pub fn new(method: &'a ImplItemMethod, server_type: &'a Type) -> syn::Result<Self> {
        validate_method(method)?;

        let sig = &method.sig;
        // Remove the initial self type from this list of args (and check it's not a typed receiver)
        let args: Vec<&FnArg> = if let Some(rec) = sig.receiver() {
            if let FnArg::Typed(_) = rec {
                return Err(syn::Error::new_spanned(
                &sig.receiver(),
                "Self receivers with a specified type, such as self: Box<Self>, are not supported by phalanx_server."
                ));
            }
            sig.inputs.iter().skip(1).collect()
        } else {
            sig.inputs.iter().collect()
        };

        let mut attrs = Vec::with_capacity(method.attrs.len() - 1);
        let mut route_attr = None;
        for attr in &method.attrs {
            match RouteAttr::try_from(attr) {
                Ok(parsed_attr) => {
                    if route_attr.is_some() {
                        return Err(syn::Error::new_spanned(
                            &attr,
                            "Multiple route attributes is not supported",
                        ));
                    }
                    route_attr = Some(parsed_attr);
                }
                Err(_) => attrs.push(attr),
            }
        }

        Ok(Self {
            method,
            server_type,
            args,
            attrs,
            route_attr: route_attr.unwrap(),
        })
    }

    pub fn fn_name(&self) -> &Ident {
        &self.method.sig.ident
    }
}

impl<'a> ToTokens for Route<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let arg_names: Vec<&Ident> = self.args.iter().map(|f| match f {
            FnArg::Typed(typed) => {
                match typed.pat.as_ref() {
                    Pat::Ident(pat_ident) => {
                        &pat_ident.ident
                    },
                    pat => panic!("Unknown pattern: {:?}", pat),
                }
            },
            FnArg::Receiver(_) => panic!("Receiver type found when it should have been automatically removed from arg list already.")
        }).collect();

        let arg_types: Vec<&Type> = self.args.iter().map(|f| match f {
            FnArg::Typed(typed) => typed.ty.as_ref(),
            FnArg::Receiver(_) => panic!("Receiver type found when it should have been automatically removed from arg list already.")
        }).collect();

        let asyncness = self.method.sig.asyncness;
        let awaitness = match asyncness {
            Some(_) => quote! {.await},
            None => quote! {},
        };

        let fn_name = self.fn_name();
        let ret_type = &self.method.sig.output;
        let server_type = self.server_type;
        let RouteAttr { path, route } = &self.route_attr;
        let attrs = &self.attrs;

        let stream = quote! {
            #[actix_web::#path(#route)]
            #(#attrs)*
            pub #asyncness fn #fn_name ( server: phalanx::reexports::web::Data<#server_type>, phalanx::reexports::web::Path(( #(#arg_names),* )): phalanx::reexports::web::Path<( #(#arg_types),* )> ) #ret_type {
                server.into_inner(). #fn_name ( #(#arg_names),* ) #awaitness
            }
        };

        tokens.extend(stream);
    }
}

fn validate_method(method: &ImplItemMethod) -> syn::Result<()> {
    if method.defaultness.is_some() {
        return Err(syn::Error::new_spanned(
            &method.defaultness,
            "Default methods are not currently supported by phalanx_server.",
        ));
    }

    let sig = &method.sig;
    if sig.unsafety.is_some() {
        return Err(syn::Error::new_spanned(
            &sig.abi,
            "Unsafe methods are not supported by phalanx_server.",
        ));
    }

    let sig = &method.sig;
    if sig.abi.is_some() {
        return Err(syn::Error::new_spanned(
            &sig.abi,
            "ABI methods are not supported by phalanx_server.",
        ));
    }
    if sig.variadic.is_some() {
        return Err(syn::Error::new_spanned(
            &sig.variadic,
            "Variadic methods are not supported by phalanx_server.",
        ));
    }
    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "Generic methods are not supported by phalanx_server.",
        ));
    }
    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "Generic methods are not supported by phalanx_server.",
        ));
    }

    Ok(())
}

mod route_attr {
    use std::convert::TryFrom;

    use super::*;

    pub(super) struct RouteAttr {
        pub path: Path,
        pub route: LitStr,
    }

    impl TryFrom<&Attribute> for RouteAttr {
        type Error = syn::Error;

        fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
            #[derive(Debug)]
            struct RawRoute {
                route: LitStr,
            }

            impl Parse for RawRoute {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    Ok(Self {
                        route: input.parse()?,
                    })
                }
            }

            let path = attr.path.clone();
            let RawRoute { route } = attr.parse_args::<RawRoute>()?;

            Ok(Self { path, route })
        }
    }
}
