use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use tokio::time::sleep;
use ultimate_control::async_driver::{AsyncUcDriver, DEFAULT_ADDR};
use ultimate_control::params;

/// Talk to a PreSonus device through Universal Control's UCNET server.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Address of the UCNET server (Universal Control on this machine).
    #[arg(long, default_value = DEFAULT_ADDR)]
    addr: String,

    /// Print every packet sent and received.
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Subscribe and print all traffic, reconnecting when the link drops (default).
    Monitor,
    /// Print every parameter path and value the device reports.
    DumpState,
    /// Print a single parameter from the device state.
    Get { param: String },
    /// Set any parameter, e.g. `set main/ch1/hardwareMute 1`.
    Set { param: String, value: f32 },
    /// Software mute of the main bus (main/ch1/mute).
    MainMute { state: Switch },
    /// The mute the physical button toggles (main/ch1/hardwareMute).
    HardwareMute { state: Switch },
    /// Mute line channel 1.
    Ch1Mute { state: Switch },
}

#[derive(Clone, Copy, ValueEnum)]
enum Switch {
    On,
    Off,
}

impl From<Switch> for f32 {
    fn from(s: Switch) -> f32 {
        match s {
            Switch::On => 1.0,
            Switch::Off => 0.0,
        }
    }
}

const ECHO_WAIT: Duration = Duration::from_secs(2);
const STATE_WAIT: Duration = Duration::from_secs(5);
const RECONNECT_DELAY: Duration = Duration::from_secs(1);

async fn connect(cli: &Cli) -> std::io::Result<AsyncUcDriver> {
    let mut driver = AsyncUcDriver::connect(&cli.addr).await?;
    driver.trace = cli.verbose;
    driver.subscribe().await?;
    Ok(driver)
}

async fn set(cli: &Cli, param: &str, value: f32) -> std::io::Result<()> {
    let mut driver = connect(cli).await?;
    let echoed = driver.set_param_confirmed(param, value, ECHO_WAIT).await?;
    println!("{param} = {echoed} (confirmed by device)");
    Ok(())
}

async fn monitor(cli: &Cli) {
    loop {
        match AsyncUcDriver::connect(&cli.addr).await {
            Ok(mut driver) => {
                driver.trace = true;
                println!("Connected to {}", cli.addr);
                if let Err(e) = driver.subscribe().await {
                    println!("Subscribe failed: {e}");
                } else {
                    driver.read_response().await;
                }
                println!("Disconnected, retrying in {RECONNECT_DELAY:?}");
            }
            Err(e) => println!(
                "Connect to {} failed: {e}, retrying in {RECONNECT_DELAY:?}",
                cli.addr
            ),
        }
        sleep(RECONNECT_DELAY).await;
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command.as_ref().unwrap_or(&Command::Monitor) {
        Command::Monitor => monitor(&cli).await,
        Command::DumpState => {
            let mut driver = connect(&cli).await?;
            for p in driver.wait_for_state(STATE_WAIT).await? {
                println!("{} = {}", p.path, p.value);
            }
        }
        Command::Get { param } => {
            let mut driver = connect(&cli).await?;
            let state = driver.wait_for_state(STATE_WAIT).await?;
            match state.iter().find(|p| &p.path == param) {
                Some(p) => println!("{} = {}", p.path, p.value),
                None => {
                    eprintln!("{param} not found; `dump-state` lists everything");
                    std::process::exit(1);
                }
            }
        }
        Command::Set { param, value } => set(&cli, param, *value).await?,
        Command::MainMute { state } => set(&cli, params::MAIN_MUTE, (*state).into()).await?,
        Command::HardwareMute { state } => {
            set(&cli, params::HARDWARE_MUTE, (*state).into()).await?
        }
        Command::Ch1Mute { state } => set(&cli, params::CH1_MUTE, (*state).into()).await?,
    }

    Ok(())
}
