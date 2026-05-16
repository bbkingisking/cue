use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};
use dirs::data_dir;
use thiserror::Error;
use crate::musicbrainz::MusicBrainzRelease;

use crate::{config::Artist};

const STATE_FILENAME: &str = "state.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub artists: HashMap<String, Vec<MusicBrainzRelease>>,
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
        let empty_state = State { artists: HashMap::new() };
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

    pub fn update_existing_artist(&mut self, artist: &Artist, releases: Vec<MusicBrainzRelease>) {
        let existing_releases = self.artists.get_mut(&artist.mbid).expect("artist should exist in state after partition");
        existing_releases.extend(releases.iter().cloned());

    }

    pub fn insert_new_artist(&mut self, artist: &Artist, releases: Vec<MusicBrainzRelease>) {
        self.artists.insert(artist.mbid.clone(), releases);
    }

    pub fn persist(&self) -> Result<(), StateError> {
        let dir = Self::path()?;
        let final_path = dir.join(STATE_FILENAME);
        let tmp_file = dir.join(format!("{}.tmp", STATE_FILENAME));
        let serialized = serde_json::to_string(&self)?;
        std::fs::write(&tmp_file, &serialized)?;
        std::fs::rename(&tmp_file, &final_path)?;
        Ok(())
    }
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
}
