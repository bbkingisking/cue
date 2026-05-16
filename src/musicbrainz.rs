use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    primary_type_id: Option<String>,
}

pub fn fetch_releases(artist_mbid: &str) -> Result<Vec<MusicBrainzRelease>, NetworkError> {
    let url = format!(
        "https://musicbrainz.org/ws/2/release-group?artist={}&fmt=json&limit=100",
        artist_mbid
    );

    let musicbrainz_response = ureq::get(url)
        .call()
        .map_err(NetworkError::RequestFailed)?
        .body_mut()
        .read_json::<MusicBrainzResponse>()
        .map_err(NetworkError::DeserializationFailed)?;

    Ok(musicbrainz_response.release_groups)
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
