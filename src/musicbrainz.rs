use serde::{Deserialize, Serialize};

use thiserror::Error;

pub fn fetch_releases(_aritst_mbid: &str) -> Result<Vec<MusicBrainzRelease>, NetworkError> {
    todo!();
}

pub fn diff(local: &[MusicBrainzRelease], fresh: &[MusicBrainzRelease]) -> Vec<MusicBrainzRelease> {
    let delta: Vec<MusicBrainzRelease> = fresh
        .iter()
        .filter(|r| !local.iter().any(|k| k.id == r.id))
        .cloned()
        .collect();

    delta
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
pub enum NetworkError { }
