use diesel::{
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};
use diesel_example::BlogServer;

use phalanx::prelude::*;

use actix_web::{middleware, App, HttpServer};

#[phalanx::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,phalanx=info");
    env_logger::init();

    let manager = ConnectionManager::<SqliteConnection>::new("test.db");

    let pool = r2d2::Pool::builder().build(manager)?;

    let server = BlogServer::new(pool);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .phalanx_mount(server.clone())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
