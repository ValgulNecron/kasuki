mod config_structure;
mod event_handler;

use anyhow::{Context, Result};

#[tokio::main]

fn main() -> Result<()> {

    #[cfg(unix)]
    {
        // wait for exit signal
    }

    Ok(())
}
