use ultimate_control::driver::*;

fn main() -> std::io::Result<()> {
    if let Ok(mut driver) = UcDriver::new() {

        println!("Connected to the server!");

        driver.subscribe()?;
        driver.ch1_mute(true)?;

        loop {
            driver.read_response()?;
        }
    } else {
        println!("Couldn't connect to server...");
    }

    Ok(())
}

