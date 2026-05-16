use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{Artist, ReleaseType};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct MusicBrainzResponse {
    pub release_groups: Vec<MusicBrainzRelease>,
    pub release_group_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MusicBrainzRelease {
    pub title: String,
    id: String,
    first_release_date: String,
    primary_type: Option<String>,
}

const PAGE_SIZE: u32 = 100;

pub fn fetch_releases(artist_mbid: &str) -> Result<Vec<MusicBrainzRelease>, NetworkError> {
    let mut all_releases: Vec<MusicBrainzRelease> = Vec::new();
    let mut offset: u32 = 0;

    loop {
        let url = format!(
            "https://musicbrainz.org/ws/2/release-group?artist={}&fmt=json&limit={}&offset={}",
            artist_mbid, PAGE_SIZE, offset
        );

        let page = ureq::get(url)
            .header("User-Agent", "cue (https://github.com/bbkingisking/cue)")
            .call()
            .map_err(NetworkError::RequestFailed)?
            .body_mut()
            .read_json::<MusicBrainzResponse>()
            .map_err(NetworkError::DeserializationFailed)?;

        let total = page.release_group_count;
        all_releases.extend(page.release_groups);
        offset += PAGE_SIZE;

        if offset >= total {
            break;
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(all_releases)
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
