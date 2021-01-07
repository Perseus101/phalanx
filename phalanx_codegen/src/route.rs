use std::convert::TryFrom;

use proc_macro2::{Ident, TokenStream as TokenStream2};

use syn::{parse::Parse, Attribute, FnArg, ImplItemMethod, LitStr, Pat, Path, Type};

use quote::{quote, ToTokens};

use route_attr::RouteAttr;

pub struct Route<'a> {
    method: &'a ImplItemMethod,
    server_type: &'a Type,
    attrs: Vec<&'a Attribute>,
    route_attr: RouteAttr,
}

impl<'a> Route<'a> {
    pub fn new(method: &'a ImplItemMethod, server_type: &'a Type) -> syn::Result<Self> {
        validate_method(method)?;

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
            attrs,
            route_attr: route_attr.unwrap(),
        })
    }

    pub fn fn_name(&self) -> &Ident {
        &self.method.sig.ident
    }
}

pub struct ServerRoute<'a>(Route<'a>);

impl<'a> From<Route<'a>> for ServerRoute<'a> {
    fn from(route: Route<'a>) -> Self {
        ServerRoute(route)
    }
}

impl<'a> ToTokens for ServerRoute<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Get the method arguments, but skip the self parameter
        let args: Vec<&FnArg> = self.0.method.sig.inputs.iter().skip(1).collect();

        // Extract the arg names
        let arg_names: Vec<&Ident> = args.iter().map(|f| match f {
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

        // Extract the arg types
        let arg_types: Vec<&Type> = args.iter().map(|f| match f {
            FnArg::Typed(typed) => typed.ty.as_ref(),
            FnArg::Receiver(_) => panic!("Receiver type found when it should have been automatically removed from arg list already.")
        }).collect();

        // Output the new method
        let fn_name = self.0.fn_name();
        let ret_type = &self.0.method.sig.output;
        let server_type = self.0.server_type;
        let RouteAttr { path, route } = &self.0.route_attr;
        let attrs = &self.0.attrs;

        let stream = quote! {
            #[actix_web::#path(#route)]
            #(#attrs)*
            pub async fn #fn_name ( server: phalanx::reexports::web::Data<#server_type>, phalanx::reexports::web::Path(( #(#arg_names),* )): phalanx::reexports::web::Path<( #(#arg_types),* )> ) #ret_type {
                server.into_inner(). #fn_name ( #(#arg_names),* ).await
            }
        };

        tokens.extend(stream);
    }
}

pub struct ClientRoute<'a>(Route<'a>);

impl<'a> From<Route<'a>> for ClientRoute<'a> {
    fn from(route: Route<'a>) -> Self {
        ClientRoute(route)
    }
}

impl<'a> ToTokens for ClientRoute<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Get the method arguments
        let args: Vec<&FnArg> = self.0.method.sig.inputs.iter().collect();
        let fn_name = self.0.fn_name();
        let raw_ret_type = &self.0.method.sig.output;
        let attrs = &self.0.attrs;
        // let server_type = self.0.server_type;
        let RouteAttr { path, route } = &self.0.route_attr;

        let format_args: Vec<TokenStream2> = args
            .iter()
            .skip(1)
            .map(|arg| match arg {
                FnArg::Typed(typed) => &typed.pat,
                arg => panic!("Unknown argument type found: {:?}", arg),
            })
            .map(|ident| quote! { #ident = #ident })
            .collect();

        let format_url = if format_args.len() > 0 {
            quote! { &format!( #route , #(#format_args),* ) }
        } else {
            quote! { #route }
        };

        // Ensure the type is a result type
        let ret_type = match raw_ret_type {
            syn::ReturnType::Default => {
                quote! { () }
            }
            syn::ReturnType::Type(_, ty) => {
                quote! { #ty }
            }
        };

        let stream = quote! {
            #(#attrs)*
            pub async fn #fn_name ( #(#args),* ) -> Result< #ret_type , Box<dyn std::error::Error> > {
                let __client  = phalanx::client::PhalanxClient::client(self);
                let res = phalanx::client::PhalanxResponse::from(__client.client. #path (&__client.format_url( #format_url )).send().await?);
                Ok(<#ret_type as phalanx::util::AsyncTryFrom<phalanx::client::PhalanxResponse>>::try_from(res).await?)
            }
        };

        tokens.extend(stream);
    }
}

fn validate_method(method: &ImplItemMethod) -> syn::Result<()> {
    if method.defaultness.is_some() {
        return Err(syn::Error::new_spanned(
            &method.defaultness,
            "Default methods are not currently supported by phalanx.",
        ));
    }

    let sig = &method.sig;

    match sig.receiver() {
        None => return Err(syn::Error::new_spanned(
        &sig,
        "Phalanx methods must have a &self parameter."
        )),
        Some(FnArg::Typed(_)) => return Err(syn::Error::new_spanned(
        &sig.receiver(),
        "Self receivers with a specified type, such as self: Box<Self>, are not supported by phalanx."
        )),
        _ => {}
    }

    if sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            &sig,
            "Phalanx methods must be async.",
        ));
    }

    if sig.unsafety.is_some() {
        return Err(syn::Error::new_spanned(
            &sig.unsafety,
            "Unsafe methods are not supported by phalanx.",
        ));
    }

    if sig.abi.is_some() {
        return Err(syn::Error::new_spanned(
            &sig.abi,
            "ABI methods are not supported by phalanx.",
        ));
    }
    if sig.variadic.is_some() {
        return Err(syn::Error::new_spanned(
            &sig.variadic,
            "Variadic methods are not supported by phalanx.",
        ));
    }
    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "Generic methods are not supported by phalanx.",
        ));
    }
    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "Generic methods are not supported by phalanx.",
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
