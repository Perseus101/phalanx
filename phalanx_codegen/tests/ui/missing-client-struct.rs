use phalanx::client::Client as InnerClient;
use phalanx_codegen::PhalanxClient;

#[derive(PhalanxClient)]
struct Client {
    client: InnerClient,
}

fn main() {}
