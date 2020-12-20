use phalanx::prelude::*;

#[derive(Clone)]
pub struct SimpleServer {}

#[phalanx_server]
impl SimpleServer {
    #[get("/{id}/{name}/index.html")]
    async fn index(&self, id: u32, name: String) -> String {
        format!("Hello {}! id:{}", name, id)
    }

    #[get("/")]
    fn root(&self) -> HttpResponse {
        HttpResponse::Found()
            .header(http::header::LOCATION, "/0/World/index.html")
            .finish()
    }
}

// #[get("/{id}/{name}/index.html")]
// async fn index(
//     data: web::Data<SimpleServer>,
//     web::Path((id, name)): web::Path<(u32, String)>,
// ) -> impl Responder {
//     data.into_inner().index(id, name)
// }

// impl PhalanxServer for SimpleServer {
//     fn mount(config: &mut web::ServiceConfig) {
//         config.service(index);
//     }
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let server = SimpleServer {};

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(SimpleServer::mount)
            .data(server.clone())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
