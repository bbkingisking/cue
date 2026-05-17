use crate::musicbrainz::MusicBrainzRelease;

pub const DEFAULT_FORMAT: &str = "%A - %T - %D";

pub fn format_release(template: &str, artist: &str, release: &MusicBrainzRelease) -> String {
    let release_type = release.primary_type
        .as_deref()
        .unwrap_or("Unknown release type");

    template
        .replace("%A", artist)
        .replace("%T", &release.title)
        .replace("%D", &release.first_release_date)
        .replace("%I", &release.id)
        .replace("%R", release_type)
}

pub fn print_output(new_releases: &[(String, MusicBrainzRelease)], format: &str) {
    for (artist_name, release) in new_releases {
        println!("{}", format_release(format, artist_name, release));
    }
}