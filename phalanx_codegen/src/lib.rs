use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;

mod route;
mod service;

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
pub fn phalanx(attr: TokenStream, input: TokenStream) -> TokenStream {
    service::Service::from_tokens(attr, input)
}
