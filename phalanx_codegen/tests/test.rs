use phalanx_codegen::{get, phalanx, PhalanxClient};

mod noargs {
    use super::*;
    use phalanx::client::Client;

    #[derive(Clone)]
    struct PayloadServer;

    #[derive(PhalanxClient)]
    struct NoArgClient(#[client] Client);

    #[phalanx(NoArgClient)]
    impl PayloadServer {
        #[get("/")]
        async fn index(&self) {}
    }

    fn _test() {
        let client = NoArgClient(Client::url("http://localhost:8080"));
        let _future = client.index();
    }
}

mod payload {
    use super::*;
    use phalanx::client::Client;

    #[derive(Clone)]
    struct PayloadServer;

    #[derive(PhalanxClient)]
    struct PayloadClient(#[client] Client);

    #[phalanx(PayloadClient)]
    impl PayloadServer {
        #[get("/{path}")]
        async fn index(&self, path: i32, _payload: ()) {
            println!("Path: {:?}", path);
        }
    }

    fn _test() {
        let client = PayloadClient(Client::url("http://localhost:8080"));
        let _future = client.index(0, ());
    }
}
