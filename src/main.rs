use clap::{Args, Parser, Subcommand};
use crate::uti::Uti;

mod changelog;
mod model;
mod uti;
mod util;
mod db;

/// Export photos from the macOS Photos library, organized by album and/or date.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {

    /// Print the changelog
    Changelog,

    /// List all albums in the library
    ListAlbums(ListAlbumsArgs),

    /// Export assets from the library to a given location
    Export(ExportArgs)
}

#[derive(Args, Debug)]
pub struct ListAlbumsArgs {

    /// Path to the Photos library
    library_path: String,
}

#[derive(Args, Debug)]
pub struct ExportArgs {

    /// Path to the Photos library
    library_path: String,

    /// Output directory
    output_dir: String,

    /// Group assets by album
    #[arg(short = 'a', long = "by-album", group = "strategy")]
    album: bool,

    /// Group assets by year/month
    #[arg(short = 'm', long = "by-year-month", group = "strategy")]
    year_month: bool,

    /// Group assets by year/month/album
    #[arg(short = 'M', long = "by-year-month-album", group = "strategy")]
    year_month_album: bool,

    /// Include assets in the albums matching the given ids
    #[arg(short = 'i', long = "include-albums", group = "ids", num_args = 0.., value_delimiter = ' ')]
    include: Option<Vec<i32>>,

    /// Exclude assets in the albums matching the given ids
    #[arg(short = 'x', long = "exclude-albums", group = "ids", num_args = 1.., value_delimiter = ' ')]
    exclude: Option<Vec<i32>>,

    /// Include hidden assets
    #[arg(short = 'H', long = "include-hidden", group = "hidden")]
    include_hidden: bool,

    /// Assets must be hidden
    #[arg(long = "must-be-hidden", group = "hidden")]
    must_be_hidden: bool,

    /// Restore original filenames
    #[arg(short = 'r', long = "restore-original-filenames")]
    restore_original_filenames: bool,

    /// Flatten album structure
    #[arg(short = 'f', long = "flatten-albums")]
    flatten_albums: bool,

    /// Include edited versions of the assets if available
    #[arg(short = 'e', long = "include-edited", group = "edited")]
    include_edited: bool,

    /// Always export the edited version of an asset if available
    #[arg(short = 'E', long = "only-edited", group = "edited")]
    only_edited: bool,

    /// Dry run
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,
}


fn main() {
    //
}