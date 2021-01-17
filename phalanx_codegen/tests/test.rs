use phalanx::client::Client;
use phalanx_codegen::{get, phalanx, PhalanxClient};

mod noargs {
    use super::*;

    #[derive(Clone)]
    struct PayloadServer;

    #[derive(PhalanxClient)]
    struct NoArgClient(#[client] Client);

    #[phalanx(NoArgClient)]
    impl PayloadServer {
        #[get("/")]
        async fn index(&self) {}
    }

    // Verify the code compiles and the client methods are added
    fn _test() {
        let client = NoArgClient(Client::url("http://localhost:8080"));
        let _future = client.index();
    }
}

mod payload {
    use super::*;

    #[derive(Clone)]
    struct PayloadServer;

    #[derive(PhalanxClient)]
    struct PayloadClient(#[client] Client);

    #[phalanx(PayloadClient)]
    impl PayloadServer {
        #[get("/{path}")]
        async fn index(&self, path: i32, payload: String) {
            println!("Path: {:?} Payload: {:?}", path, payload);
        }
    }

    // Verify the code compiles and the client methods are added
    fn _test() {
        let client = PayloadClient(Client::url("http://localhost:8080"));
        let _future = client.index(0, "".into());
    }
}

mod json {
    use super::*;
    use phalanx::web;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct SimplePayload {
        data: i32,
    }

    #[derive(Clone)]
    struct PayloadServer;

    #[derive(PhalanxClient)]
    struct PayloadClient(#[client] Client);

    #[phalanx(PayloadClient)]
    impl PayloadServer {
        #[get("/{path}")]
        async fn index(&self, path: i32, payload: web::Json<SimplePayload>) {
            println!("Path: {:?} Payload: {:?}", path, payload.into_inner());
        }
    }

    // Verify the code compiles and the client methods are added
    fn _test() {
        let client = PayloadClient(Client::url("http://localhost:8080"));
        let _future = client.index(0, web::Json(SimplePayload { data: 0i32 }));
    }
}
