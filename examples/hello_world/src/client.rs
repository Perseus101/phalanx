use hello_world::SimpleClient;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SimpleClient::new("http://localhost:8080");
    println!("{}", client.foo("World".to_string()).await?);

    Ok(())
}
