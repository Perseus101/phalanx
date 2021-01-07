pub mod client;
pub mod server;
pub mod util;

pub mod prelude {
    pub use super::server::PhalanxServer;

    pub use super::reexports::*;

    pub use phalanx_codegen::{connect, delete, get, head, options, patch, post, put, trace};
    pub use phalanx_codegen::{phalanx_client, phalanx_server};
}

pub mod reexports {
    pub use actix_web::{http, middleware, web, App, HttpResponse, HttpServer, Responder};

    pub use reqwest::Client;
}
