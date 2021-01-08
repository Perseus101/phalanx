use phalanx::client::Client as InnerClient;
use phalanx_codegen::PhalanxClient;

#[derive(PhalanxClient)]
struct Client(InnerClient);

fn main() {}
