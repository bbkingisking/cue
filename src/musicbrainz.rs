use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize)]
struct MusicBrainzArtist {
    name: String,
}

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
    pub first_release_date: String,
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

        let body = ureq::get(url)
            .header("User-Agent", "cue (https://github.com/bbkingisking/cue)")
            .call()
            .map_err(NetworkError::RequestFailed)?
            .body_mut()
            .read_to_string()
            .map_err(NetworkError::RequestFailed)?;

        let page = serde_json::from_str::<MusicBrainzResponse>(&body)
            .map_err(NetworkError::DeserializationFailed)?;

        let total = page.release_group_count;
        all_releases.extend(page.release_groups);
        offset += PAGE_SIZE;

        std::thread::sleep(std::time::Duration::from_millis(1100));

        if offset >= total {
            break;
        }
    }

    Ok(all_releases)
}

pub fn fetch_artist_name(artist_mbid: &str) -> Result<String, NetworkError> {
    let url = format!(
        "https://musicbrainz.org/ws/2/artist/{}?fmt=json",
        artist_mbid
    );

    let body = ureq::get(url)
        .header("User-Agent", "cue (https://github.com/bbkingisking/cue)")
        .call()
        .map_err(NetworkError::RequestFailed)?
        .body_mut()
        .read_to_string()
        .map_err(NetworkError::RequestFailed)?;

    let artist = serde_json::from_str::<MusicBrainzArtist>(&body)
        .map_err(NetworkError::DeserializationFailed)?;

    std::thread::sleep(std::time::Duration::from_millis(1100));

    Ok(artist.name)
}

pub fn filter_releases(releases: &[MusicBrainzRelease], required: &[ReleaseType]) -> Vec<MusicBrainzRelease> {
    // If all release types are required, no need to filter
    let required_set: HashSet<&ReleaseType> = required.iter().collect();
    let all_types = Artist::all_release_types();
    let all_set: HashSet<&ReleaseType> = all_types.iter().collect();
    if required_set == all_set {
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
    let local_ids: std::collections::HashSet<&str> = local.iter().map(|r| r.id.as_str()).collect();
    fresh
        .iter()
        .filter(|r| !local_ids.contains(r.id.as_str()))
        .cloned()
        .collect()
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Could not send a request to musicbrainz's API.")]
    RequestFailed(ureq::Error),
    #[error("Could not deserialize MusicBrainz response.")]
    DeserializationFailed(#[from] serde_json::Error),
}
