mod config;
mod state;
mod musicbrainz;

use crate::{config::{Config, ConfigError}, state::{State, StateError}};

fn main() -> Result<(), anyhow::Error> {
    let conf = match Config::load() {
        Ok(c) => c,
        Err(ConfigError::ConfigNotFound) => {
            let config_path = Config::create()?;
            eprintln!("A configuration file was not found. A sample one was created in {}. Edit it and run the application again.", &config_path.to_string_lossy());
            return Ok(())
        },
        Err(e) => return Err(e.into())
    };

    let mut state = match State::load() {
        Ok(s) => s,
        Err(StateError::StateNotFound) => State::create()?,
        Err(e) => return Err(e.into())
    };

    let _new_releases = state.sync(&conf)?;
    state.persist()?;
    // iterate over new_releases and print to stdout
    Ok(())
}
