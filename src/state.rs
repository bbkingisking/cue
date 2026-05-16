use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};
use dirs::data_dir;
use thiserror::Error;

use crate::{config::Config, musicbrainz::fetch_releases};

const STATE_FILENAME: &str = "state.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    state: HashMap<String, Vec<MusicBrainzRelease>>,
}

impl State {
    fn path() -> Result<PathBuf, StateError> {
        let xdg_conf = data_dir().ok_or(StateError::DataDirNotFound)?;
        let app_conf = xdg_conf.join(env!("CARGO_PKG_NAME"));
        Ok(app_conf)
    }

    pub fn create() -> Result<State, StateError> {
        let path = Self::path()?;
        std::fs::create_dir_all(&path)?;
        let empty_state = State { state: HashMap::new() };
        let serialized = serde_json::to_string(&empty_state)?;
        std::fs::write(path.join(STATE_FILENAME), serialized)?;
        Ok(empty_state)
    }

    pub fn load() -> Result<State, StateError> {
        let path = Self::path()?.join(STATE_FILENAME);
        if !path.try_exists()? {
            return Err(StateError::StateNotFound)
        }
        let raw = std::fs::read_to_string(path)?;
        let state = serde_json::from_str(&raw)?;
        Ok(state)
    }

    pub fn sync(&mut self, config: &Config) -> Result<Vec<MusicBrainzRelease>, StateError> {
        // This fn will return a vec of all new releases
        // If there are no new ones, the vec will be empty
        // It will be up to the caller to iter over it
        let mut new_releases: Vec<MusicBrainzRelease> = Vec::new();

        let (new_artists, existing_artists): (Vec<_>, Vec<_>) = config.artists
            .iter()
            .partition(|a| !self.state.contains_key(&a.mbid));

        for artist in new_artists {
            let releases = fetch_releases(&artist.mbid)?;
            self.state.insert(artist.mbid.clone(), releases);
        }

        for artist in existing_artists {
            let releases_local = self.state.get_mut(&artist.mbid).expect("artist should exist in state after partition");
            let mbid_releases = fetch_releases(&artist.mbid)?;

            let mbid_new: Vec<_> = mbid_releases
                .into_iter()
                .filter(|r| !releases_local.iter().any(|k| k.id == r.id))
                .collect();

            releases_local.extend(mbid_new.iter().cloned());
            new_releases.extend(mbid_new);
        }

        Ok(new_releases)
    }

    pub fn persist(&self) -> Result<(), StateError> {
        let path = Self::path()?.join(STATE_FILENAME);
        let serialized = serde_json::to_string(&self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct MusicBrainzResponse {
    pub release_groups: Vec<MusicBrainzRelease>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MusicBrainzRelease {
    title: String,
    id: String,
    first_release_date: String,
    primary_type: Option<String>,
    primary_type_id: Option<String>,
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Could not find data dir.")]
    DataDirNotFound,
    #[error("Could not access state file.")]
    StateInaccessible(#[from] std::io::Error),
    #[error("Could not deserialize state file.")]
    StateCouldNotDeserialize(#[from] serde_json::Error),
    #[error("Could not find state.json")]
    StateNotFound,
    #[error("Could not send a request to musicbrainz API.")]
    NetworkError(#[from] ureq::Error),
}
