use crowsi_network_sandbox::{IsolationBudget, IsolationCapabilityV1, isolated_command};
use std::{error::Error, io, path::Path};

fn main() {
    if let Err(error) = run() {
        eprintln!("network sandbox failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1).as_deref() {
        None | Some("inspect") => {
            serde_json::to_writer_pretty(io::stdout(), &IsolationCapabilityV1::default())?;
            println!();
        }
        Some("probe") => {
            let status = isolated_command(Path::new("/usr/bin/true"), &[], IsolationBudget::Short)?
                .status()?;
            if !status.success() {
                return Err("Linux user/network namespace probe was rejected".into());
            }
            serde_json::to_writer_pretty(io::stdout(), &IsolationCapabilityV1::default())?;
            println!();
        }
        Some("help" | "-h" | "--help") => {
            println!("usage: crowsi-network-sandbox [inspect|probe]");
        }
        Some(_) => return Err("expected inspect or probe".into()),
    }
    Ok(())
}
