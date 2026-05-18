mod cli;
mod config;
mod state;
mod musicbrainz;
mod output;

use clap::Parser;
use log::info;

use crate::{cli::Cli, config::{Config, ConfigError}, musicbrainz::{MusicBrainzRelease, diff, fetch_artist_name, fetch_releases, filter_releases}, output::print_output, state::{State, StateError}};

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("cue=info")
        ).init();
    } else {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("warn")
        ).init();
    }

    info!("Desired output format: {}", &cli.format);

    let conf = match Config::load() {
        Ok(c) => c,
        Err(ConfigError::ConfigNotFound) => {
            let config_path = Config::create()?;
            eprintln!("A configuration file was not found. A sample one was created in {}. Edit it and run the application again.", &config_path.to_string_lossy());
            return Ok(())
        },
        Err(e) => return Err(e.into())
    };
    info!("Config loaded. Tracking {} artist(s).", &conf.artists.len());

    let mut state = match State::load() {
        Ok(s) => s,
        Err(StateError::StateNotFound) => State::create()?,
        Err(e) => return Err(e.into())
    };
    info!("Local state loaded.");

    let mut new_releases: Vec<(String, MusicBrainzRelease)> = Vec::new();

    let (new_artists, existing_artists): (Vec<_>, Vec<_>) = conf.artists
        .iter()
        .partition(|a| !state.artists.contains_key(&a.mbid));

    if !new_artists.is_empty() {
        info!("{} new artists in config.", &new_artists.len());
    }

    // These need their own branch since we don't want to print out every release
    for artist in new_artists {
        let name = fetch_artist_name(&artist.mbid)?;
        let all_releases = fetch_releases(&artist.mbid)?;
        let filtered_releases = filter_releases(&all_releases, &artist.release_types);
        info!("Fetched {} releases for {}, persisting them to locaal state.", &filtered_releases.len(), &name);
        state.insert_new_artist(artist, name, filtered_releases);
    }

    for artist in existing_artists {
        let all_releases = fetch_releases(&artist.mbid)?;
        let filtered_releases = filter_releases(&all_releases, &artist.release_types);

        let local = state.artists.get(&artist.mbid).expect("Artist should exist at this point.");
        let delta = diff(&local.releases, &filtered_releases);

        info!("Fetched {} releases for {}, {} of them are new.", &all_releases.len(), &local.name, &delta.len());
        let artist_name = local.name.clone();
        let named_delta: Vec<(String, MusicBrainzRelease)> = delta.iter()
            .map(|r| (artist_name.clone(), r.clone()))
            .collect();

        new_releases.extend(named_delta);
        state.update_existing_artist(artist, delta);
    }

    state.persist()?;
    print_output(&new_releases, &cli.format);

    Ok(())
}
