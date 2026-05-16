use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{Artist, ReleaseType};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct MusicBrainzResponse {
    pub release_groups: Vec<MusicBrainzRelease>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MusicBrainzRelease {
    pub title: String,
    id: String,
    first_release_date: String,
    primary_type: Option<String>,
}

pub fn fetch_releases(artist_mbid: &str) -> Result<Vec<MusicBrainzRelease>, NetworkError> {
    let url = format!(
        "https://musicbrainz.org/ws/2/release-group?artist={}&fmt=json&limit=100",
        artist_mbid
    );

    let musicbrainz_response = ureq::get(url)
        .header("User-Agent", "cue (https://github.com/bbkingisking/cue)")
        .call()
        .map_err(NetworkError::RequestFailed)?
        .body_mut()
        .read_json::<MusicBrainzResponse>()
        .map_err(NetworkError::DeserializationFailed)?;

    Ok(musicbrainz_response.release_groups)
}

pub fn filter_releases(releases: &[MusicBrainzRelease], required: &[ReleaseType]) -> Vec<MusicBrainzRelease> {
    // If all release types are required, no need to filter
    if required == Artist::all_release_types() {
        return releases.to_vec()
    }

    // Filter for the required release types
    let filtered_releases: Vec<MusicBrainzRelease> = releases
        .to_owned()
        .into_iter()
        .filter(|g| {
            g.primary_type.as_deref()
                .map(|pt| required.iter().any(|rt| rt.as_str() == pt))
                .unwrap_or(false)
        })
        .collect();

    filtered_releases
}

pub fn diff(local: &[MusicBrainzRelease], fresh: &[MusicBrainzRelease]) -> Vec<MusicBrainzRelease> {
    let delta: Vec<MusicBrainzRelease> = fresh
        .iter()
        .filter(|r| !local.iter().any(|k| k.id == r.id))
        .cloned()
        .collect();

    delta
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Could not send a request to musicbrainz's API.")]
    RequestFailed(#[from] ureq::Error),
    #[error("Could not deserialize MusicBrainz response.")]
    DeserializationFailed(ureq::Error),
}
