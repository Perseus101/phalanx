use std::convert::TryFrom;

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};

use syn::{parse::Parse, Attribute, FnArg, ImplItemMethod, LitStr, Pat, PatType, ReturnType, Type};

use quote::{quote, ToTokens};

use regex::Regex;

use route_attr::RouteAttr;

#[derive(Clone)]
pub struct Route {
    server_type: Type,
    ident: Ident,

    args: Vec<PatType>,
    path_args: Vec<PatType>,
    payload_arg: Option<PatType>,
    ret_type: ReturnType,
    attrs: Vec<Attribute>,
    route_attr: RouteAttr,
}

impl Route {
    pub fn new(method: &ImplItemMethod, server_type: &Type) -> syn::Result<Self> {
        validate_method(method)?;

        // Get the method arguments, but  the self parameter
        let args: Vec<_> = method.sig.inputs.iter().skip(1).map(|f| match f {
            FnArg::Typed(typed) => typed.clone(),
            FnArg::Receiver(_) => panic!("Receiver type found when it should have been automatically removed from arg list already.")
        }).collect();

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
                Err(_) => attrs.push(attr.clone()),
            }
        }

        let route_attr = route_attr
            .ok_or_else(move || syn::Error::new_spanned(&method.sig, "Missing route attribute"))?;

        // Find which arguments are path arguments and determine if there is an extra payload argument
        lazy_static::lazy_static! {
            static ref RE: Regex = Regex::new(&r"\{([[:alpha:]_]+)\}").unwrap();
        }

        let path_str = route_attr.route.value();

        let mut path_arg_names: Vec<&str> = Vec::new();
        for arg_name in RE.captures_iter(&path_str) {
            let arg_name = arg_name.get(1).unwrap().as_str();
            path_arg_names.push(arg_name);
        }

        let mut payload_arg = None;
        let mut path_args = Vec::new();

        fn contains_ident(names: &Vec<&str>, ident: &Ident) -> bool {
            for name in names.iter() {
                if ident == name {
                    return true;
                }
            }
            false
        }

        // Split args into path_args and payload_arg
        for arg in args.iter() {
            match arg.pat.as_ref() {
                Pat::Ident(pat_ident) => {
                    if contains_ident(&path_arg_names, &pat_ident.ident) {
                        path_args.push(arg.clone());
                    } else {
                        if payload_arg.is_some() {
                            return Err(syn::Error::new_spanned(
                                &arg,
                                "Multiple unmatched path args",
                            ));
                        }
                        payload_arg = Some(arg.clone());
                    }
                }
                pat => panic!("Unknown pattern: {:?}", pat),
            }
        }

        Ok(Self {
            server_type: server_type.clone(),
            ident: method.sig.ident.clone(),
            args,
            path_args,
            payload_arg,
            ret_type: method.sig.output.clone(),
            attrs,
            route_attr,
        })
    }
}

pub struct ServerRoute(Route);

impl From<Route> for ServerRoute {
    fn from(route: Route) -> Self {
        ServerRoute(route)
    }
}

impl ToTokens for ServerRoute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Extract the identifier and type from each argument
        let args = split_args(&self.0.args);

        let arg_names = args.iter().map(|(ident, _)| ident);

        // Filter out the payload argument, if present
        let path_args: Vec<_> = split_args(&self.0.path_args);

        let (ret_type, ret_trailer) = match &self.0.ret_type {
            syn::ReturnType::Default => (
                quote! { -> phalanx::server::UnitResponder },
                quote! { phalanx::server::UnitResponder },
            ),
            ty => (quote! { #ty }, quote! { res }),
        };

        // Output the new method
        let fn_name = &self.0.ident;
        let fn_name_str = &self.0.ident.to_string();
        let server_type = &self.0.server_type;
        let route = &self.0.route_attr.route;
        let method = self.0.route_attr.method_ident();
        let attrs = &self.0.attrs;

        let path_args = if path_args.len() > 0 {
            let arg_names = path_args.iter().map(|(ident, _)| ident);
            let arg_types = path_args.iter().map(|(_, ty)| ty);
            quote! { phalanx::reexports::web::Path(( #(#arg_names),* )): phalanx::reexports::web::Path<( #(#arg_types),* )>, }
        } else {
            quote! {}
        };

        let payload_arg = &self.0.payload_arg;

        let stream = quote! {
            #(#attrs)*
            async fn #fn_name ( server: phalanx::reexports::web::Data<#server_type>, #path_args #payload_arg ) #ret_type {
                let res = server.into_inner(). #fn_name ( #(#arg_names),* ).await;
                #ret_trailer
            }

            let __resource = phalanx::reexports::Resource::new(#route)
                .name(#fn_name_str)
                .guard(phalanx::reexports::guard:: #method ())
                .to(#fn_name);
            __config.service(__resource);
        };

        tokens.extend(stream);
    }
}

pub struct ClientRoute(Route);

impl From<Route> for ClientRoute {
    fn from(route: Route) -> Self {
        ClientRoute(route)
    }
}

impl ToTokens for ClientRoute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let args = &self.0.args;
        let fn_name = &self.0.ident;
        let raw_ret_type = &self.0.ret_type;
        let attrs = &self.0.attrs;
        let route = &self.0.route_attr.route;
        let method = self.0.route_attr.method_ident_lower();

        // Get just the arguments which affect the path of the request
        let path_args = split_args(&self.0.path_args);

        let format_args: Vec<TokenStream2> = path_args
            .iter()
            .map(|(ident, _)| quote! { #ident = #ident })
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

        let (content_type, payload) = if let Some(payload) = &self.0.payload_arg {
            match payload.pat.as_ref() {
                Pat::Ident(ident) => (
                    quote! {
                        let __content_type = phalanx::client::ContentType::from(&#ident);
                    },
                    quote! {
                        .header("content-type", __content_type.header_value())
                        .body({
                            let body: phalanx::reexports::Body = std::convert::TryFrom::try_from(#ident)?;
                            body
                        })
                    },
                ),
                pat => panic!("Unknown pattern: {:?}", pat),
            }
        } else {
            (quote! {}, quote! {})
        };

        let stream = quote! {
            #(#attrs)*
            pub async fn #fn_name ( &self, #(#args),* ) -> Result< #ret_type , Box<dyn std::error::Error> > {
                let __client  = phalanx::client::PhalanxClient::client(self);
                #content_type
                let __req = __client.client. #method (&__client.format_url( #format_url )) #payload;
                let __res = phalanx::client::PhalanxResponse::from(__req.send().await?);
                Ok(<#ret_type as phalanx::util::AsyncTryFrom<phalanx::client::PhalanxResponse>>::try_from(__res).await?)
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

fn split_args<'a>(args: &'a Vec<PatType>) -> Vec<(&'a Ident, &'a Type)> {
    args.iter()
        .map(|typed| match typed.pat.as_ref() {
            Pat::Ident(pat_ident) => (&pat_ident.ident, typed.ty.as_ref()),
            pat => panic!("Unknown pattern: {:?}", pat),
        })
        .collect()
}
mod route_attr {
    use std::convert::TryFrom;

    use syn::spanned::Spanned;

    use super::*;

    macro_rules! method_type {
        (
            $($variant:ident, $lower:ident,)+
        ) => {
            #[derive(Debug, PartialEq, Eq, Hash, Clone)]
            pub(super) enum MethodType {
                $(
                    $variant,
                )+
            }

            impl MethodType {
                fn as_str(&self) -> &'static str {
                    match self {
                        $(Self::$variant => stringify!($variant),)+
                    }
                }

                fn as_lower_str(&self) -> &'static str {
                    match self {
                        $(Self::$variant => stringify!($lower),)+
                    }
                }


                fn parse(method: &str) -> Result<Self, String> {
                    match method {
                        $(stringify!($lower) => Ok(Self::$variant),)+
                        _ => Err(format!("Unexpected HTTP method: `{}`", method)),
                    }
                }
            }
        };
    }

    method_type! {
        Get,       get,
        Post,      post,
        Put,       put,
        Delete,    delete,
        Head,      head,
        Connect,   connect,
        Options,   options,
        Trace,     trace,
        Patch,     patch,
    }

    #[derive(Clone)]
    pub(super) struct RouteAttr {
        pub method: MethodType,
        pub route: LitStr,
        span: Span,
    }

    impl RouteAttr {
        pub fn method_ident(&self) -> Ident {
            Ident::new(self.method.as_str(), self.span)
        }

        pub fn method_ident_lower(&self) -> Ident {
            Ident::new(self.method.as_lower_str(), self.span)
        }
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

            let method_string = &attr.path.segments[attr.path.segments.len() - 1]
                .ident
                .to_string();

            let method = MethodType::parse(method_string)
                .map_err(|err| syn::Error::new_spanned(attr, err))?;
            let RawRoute { route } = attr.parse_args::<RawRoute>()?;
            Ok(Self {
                method,
                route,
                span: attr.span(),
            })
        }
    }
}