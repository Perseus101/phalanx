use std::convert::TryFrom;

use proc_macro2::{Ident, Span};

use syn::{parse::Parse, Attribute, FnArg, ImplItemMethod, LitStr, Pat, PatType, ReturnType, Type};

use regex::Regex;

pub mod client;
pub mod server;

mod route_attr;
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
