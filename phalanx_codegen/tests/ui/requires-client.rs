use phalanx_codegen::{phalanx};

#[derive(Clone)]
struct Server;

struct Client;

#[phalanx]
impl Server {
    #[get("/")]
    async fn index(&self) {}
}

fn main() {}