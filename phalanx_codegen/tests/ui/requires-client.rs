use phalanx_codegen::{phalanx_client};

#[derive(Clone)]
struct Server;

struct Client;

#[phalanx_client]
impl Server {
    #[get("/")]
    async fn index(&self) {}
}

fn main() {}