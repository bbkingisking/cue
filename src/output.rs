use crate::musicbrainz::MusicBrainzRelease;

pub fn print_output(new_releases: &[(String, MusicBrainzRelease)]) {
    for (artist_name, release) in new_releases {
        println!("{} - {} - {}", artist_name, release.title, release.first_release_date);
    }
}