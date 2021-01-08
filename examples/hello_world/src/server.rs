use hello_world::SimpleServer;

use phalanx::prelude::*;

use actix_web::{middleware, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let server = SimpleServer {};

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .phalanx_mount(server.clone())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
