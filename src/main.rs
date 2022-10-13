pub mod header;
pub mod packet;
pub mod driver;
use driver::*;

fn main() -> std::io::Result<()> {
    if let Ok(mut driver) = UcDriver::new() {

        println!("Connected to the server!");

        driver.subscribe()?;
        driver.ch1_mute(true)?;

        driver.read_response()?;
    } else {
        println!("Couldn't connect to server...");
    }

    Ok(())
}

