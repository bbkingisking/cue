use clap::Parser;
use crate::output::DEFAULT_FORMAT;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    // Output format template. Specifiers: %A (artist), %T (title), %D (date), %I (MusicBrainz ID), %R (release type)
    #[arg(short = 'f', long = "format", default_value = DEFAULT_FORMAT)]
    pub format: String,
}