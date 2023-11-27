use std::{fs::File, path::Path};

use anyhow::{Context, Result};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

pub fn create_logger(path: impl AsRef<Path>, level: LevelFilter) -> Result<()> {
    let name = path.as_ref().display().to_string();
    let error_message = format!("Failed to create log file name: {name}");

    let file = File::create(path).context(error_message)?;
    CombinedLogger::init(vec![
        TermLogger::new(
            level,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::max(), Config::default(), file),
    ])?;

    Ok(())
}
