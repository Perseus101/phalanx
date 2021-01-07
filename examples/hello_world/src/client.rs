use hello_world::SimpleClient;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = SimpleClient::new("http://localhost:8080");
    println!("{}", server.index(0, "World".to_string()).await?);

    Ok(())
}
