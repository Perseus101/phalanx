use hello_world::SimpleClient;

#[phalanx::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SimpleClient::new("http://localhost:8080");
    println!("{}", client.index("World".to_string()).await?);
    println!("{}", client.foo("World".to_string()).await?);

    Ok(())
}
