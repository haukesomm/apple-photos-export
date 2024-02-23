use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};
use crate::album_list::query_and_print_albums;

mod model;
mod album_list;
mod repo;

/// Export photos from the macOS Photos library, organized by album and/or date.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {

    /// Path of the library file
    library_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {

    /// Lists all albums in the library
    ListAlbums,

    /// Exports the specified assets from the library to a given location
    ExportAssets(ExportArgs)
}

#[derive(Args, Debug)]
struct ExportArgs {

    /// Path of the destination directory
    destination_path: String,

    /// Export photos to the root of the export directory
    #[arg(short = 'p', long = "plain", group = "strategy")]
    plain: bool,

    /// Export photos grouped by album
    #[arg(short = 'a', long = "album", group = "strategy")]
    album: bool,

    /// Export photos grouped by year/month
    #[arg(short = 'y', long = "year-month", group = "strategy")]
    year_month: bool,

    /// Export photos grouped by year/month/album
    #[arg(short = 'm', long = "year-month-album", group = "strategy")]
    year_month_album: bool,

    /// Restore original filenames
    #[arg(short = 'o', long = "restore-original-filenames")]
    restore_original_filenames: bool,

    /// Flatten albums
    #[arg(short = 'f', long = "flatten-albums")]
    flatten_albums: bool,

    /// Dry run
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,
}

fn main() {
    let args = Arguments::parse();

    let db_path_buf = PathBuf::new()
        .join(args.library_path)
        .join("database")
        .join("Photos.sqlite");

    let db_path = db_path_buf.as_path().to_str().unwrap().to_string();

    match args.command {
        Commands::ListAlbums => query_and_print_albums(&db_path),
        Commands::ExportAssets(_) => {
            println!("This command is not yet implemented.")
        }
    }
}
