mod config;
mod state;
mod musicbrainz;

use crate::{config::{Config, ConfigError}, musicbrainz::{diff, fetch_releases, MusicBrainzRelease}, state::{State, StateError}};

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

    let mut new_releases: Vec<MusicBrainzRelease> = Vec::new();

    let (new_artists, existing_artists): (Vec<_>, Vec<_>) = conf.artists
        .iter()
        .partition(|a| !state.artists.contains_key(&a.mbid));

    for artist in new_artists {
        let releases = fetch_releases(&artist.mbid)?;
        state.insert_new_artist(artist, releases);
    }

    for artist in existing_artists {
        let fresh = fetch_releases(&artist.mbid)?;
        let local = state.artists.get(&artist.mbid).expect("Artist should exist at this point.");
        let delta = diff(local, &fresh);
        new_releases.extend(delta.clone());
        state.update_existing_artist(artist, delta);
    }

    state.persist()?;

    // iterate over new_releases and print to stdout
    Ok(())
}
