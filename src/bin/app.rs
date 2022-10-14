use ultimate_control::async_driver::AsyncUcDriver;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut driver = AsyncUcDriver::new().await?;

    println!("Connected to the server!");
    driver.subscribe().await?;
    driver.ch1_mute(true).await?;

    let mut vol = 0.0f32;
    for _n in 1..10 {
        vol += 0.1;
        driver.ch1_volume(vol).await?;
        sleep(Duration::from_millis(100)).await;
    }

    driver.read_response().await;
        
    Ok(())
}

