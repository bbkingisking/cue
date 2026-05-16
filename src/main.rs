mod config;

use crate::config::{Config, ConfigError};

fn main() -> Result<(), anyhow::Error> {
    let _conf = match Config::load() {
        Ok(c) => c,
        Err(ConfigError::ConfigNotFound(path)) => {
            Config::create()?;
            return Err(ConfigError::ConfigNotFound(path).into());
        },
        Err(e) => return Err(e.into())
    };

    Ok(())
}
