use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use syn::{parse_macro_input, ImplItem, ItemImpl};

use proc_macro_error::*;

mod route;

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
    if parsed_impl.defaultness.is_some() {
        return syn::Error::new_spanned(
            &parsed_impl.defaultness,
            "Default impls are not currently supported by phalanx_server.",
        )
        .to_compile_error()
        .into();
    }

    if !parsed_impl.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &parsed_impl.generics,
            "Generics are not supported by phalanx_server.",
        )
        .to_compile_error()
        .into();
    }

    let mut routes = Vec::new();
    let server_type = parsed_impl.self_ty.as_ref();

    for item in &parsed_impl.items {
        match item {
            ImplItem::Method(method) => match route::Route::new(method, server_type) {
                Ok(route) => routes.push(route),
                Err(err) => return err.to_compile_error().into(),
            },
            ImplItem::Type(assoc_type) => {
                return syn::Error::new_spanned(
                    &assoc_type,
                    "Associated Types are not currently supported by phalanx_server.",
                )
                .to_compile_error()
                .into()
            }
            _ => (),
        }
    }

    let services: Vec<TokenStream2> = routes
        .iter()
        .map(|route| route.fn_name())
        .map(|ident| quote! { config.service( #ident ); })
        .collect();

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
