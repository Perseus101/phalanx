#![feature(type_alias_impl_trait)]

pub mod client;
pub mod server;
pub mod util;

pub use phalanx_codegen::{phalanx, PhalanxClient};

pub mod prelude {
    pub use crate::server::{mount::PhalanxMount, PhalanxServer};

    pub use phalanx_codegen::{connect, delete, get, head, options, patch, post, put, trace};
    pub use phalanx_codegen::{phalanx, PhalanxClient};
}

pub mod reexports {
    pub use actix_web::{http, middleware, web, App, HttpResponse, HttpServer, Responder};

    pub use reqwest::Client;
}
