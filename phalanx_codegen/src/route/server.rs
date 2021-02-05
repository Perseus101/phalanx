use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};

use super::Route;

pub struct ServerRoute(Route);

impl From<Route> for ServerRoute {
    fn from(route: Route) -> Self {
        ServerRoute(route)
    }
}

impl ToTokens for ServerRoute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Extract the identifier and type from each argument
        let args = super::split_args(&self.0.args);

        let arg_names = args.iter().map(|(ident, _)| ident);

        // Filter out the payload argument, if present
        let path_args: Vec<_> = super::split_args(&self.0.path_args);

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
