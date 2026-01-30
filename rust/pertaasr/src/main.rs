use std::error::Error;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let resp = reqwest::get("https://httpbin.org/ip").await?.json::<Value>().await?;
    println!("{:#?}", resp);
    Ok(())
}
