use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use syn::{parse_macro_input, ImplItem, ItemImpl, Type};

use proc_macro_error::*;

mod route;

use route::{ClientRoute, Route, ServerRoute};

macro_rules! method_macro {
    (
        $($variant:ident, $method:ident,)+
    ) => {
        $(
            #[proc_macro_attribute]
            pub fn $method(_: TokenStream, input: TokenStream) -> TokenStream {
                // These attributes are parsed and used by the phalax_server proc_macro
                input
            }
        )+
    };
}

method_macro! {
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

#[proc_macro_attribute]
#[proc_macro_error]
pub fn phalanx_server(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_impl = parse_macro_input!(input as ItemImpl);
    if let Err(err) = validate_impl(&parsed_impl) {
        return err.to_compile_error().into();
    }

    let routes = match parse_routes(&parsed_impl) {
        Ok(routes) => routes,
        Err(err) => return err.to_compile_error().into(),
    };

    let services: Vec<TokenStream2> = routes
        .iter()
        .map(|route| route.fn_name())
        .map(|ident| quote! { config.service( #ident ); })
        .collect();

    let routes: Vec<ServerRoute> = routes.into_iter().map(ServerRoute::from).collect();

    let server_type = parsed_impl.self_ty.as_ref();

    let output = quote! {
        #parsed_impl

        #(#routes)*

        impl phalanx::server::PhalanxServer for #server_type {
            fn mount(config: &mut phalanx::reexports::web::ServiceConfig) {
                #(#services)*
            }
        }
    };

    output.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn phalanx_client(attr: TokenStream, input: TokenStream) -> TokenStream {
    let client_type = parse_macro_input!(attr as Type);
    let parsed_impl = parse_macro_input!(input as ItemImpl);
    if let Err(err) = validate_impl(&parsed_impl) {
        return err.to_compile_error().into();
    }

    let routes = match parse_routes(&parsed_impl) {
        Ok(routes) => routes,
        Err(err) => return err.to_compile_error().into(),
    };

    let routes: Vec<ClientRoute> = routes.into_iter().map(ClientRoute::from).collect();

    let output = quote! {
        #parsed_impl

        impl #client_type {
            #(#routes)*
        }
    };

    output.into()
}

fn validate_impl(parsed_impl: &ItemImpl) -> syn::Result<()> {
    if parsed_impl.defaultness.is_some() {
        return Err(syn::Error::new_spanned(
            &parsed_impl.defaultness,
            "Default impls are not currently supported by phalanx_server.",
        ));
    }

    if !parsed_impl.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &parsed_impl.generics,
            "Generics are not supported by phalanx_server.",
        ));
    }

    Ok(())
}

fn parse_routes<'a>(parsed_impl: &'a ItemImpl) -> syn::Result<Vec<Route<'a>>> {
    let mut routes: Vec<Route<'a>> = Vec::new();
    let server_type = parsed_impl.self_ty.as_ref();

    for item in &parsed_impl.items {
        match item {
            ImplItem::Method(method) => match Route::new(method, server_type) {
                Ok(route) => routes.push(route),
                Err(err) => return Err(err),
            },
            ImplItem::Type(assoc_type) => {
                return Err(syn::Error::new_spanned(
                    &assoc_type,
                    "Associated Types are not currently supported by phalanx_server.",
                ));
            }
            _ => (),
        }
    }

    Ok(routes)
}
