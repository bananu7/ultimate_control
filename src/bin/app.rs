use ultimate_control::async_driver::AsyncUcDriver;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut driver = AsyncUcDriver::new().await?;

    println!("Connected to the server!");
    driver.subscribe().await?;
    driver.ch1_mute(true).await?;
    driver.read_response().await;
        
    Ok(())
}

