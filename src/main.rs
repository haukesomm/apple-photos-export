use std::fs::File;
use std::io::Write;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use colored::Colorize;
use rand::Rng;
use crate::result::{Result, Error};

mod db;
mod foundation;
mod model;
mod util;
mod result;
mod album_list;


/// Export photos from the macOS Photos library, organized by album and/or date.
#[derive(Parser, Debug)]
#[command(
    version, 
    about, 
    long_about = None, 
    after_help = "Have a look at the changelog for the latest changes:\n\
        https://github.com/haukesomm/apple-photos-export/blob/main/CHANGELOG.md"
)]
struct Arguments {
    // Path to the Photos library
    library_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print the library version
    Version,

    /// List all albums in the library
    ListAlbums,

    /// Export assets from the library to a given location
    Export(ExportArgs),
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Path to the Photos library
    //library_path: String,

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
    #[arg(short = 'i', long = "include-albums", group = "ids", num_args = 0.., value_delimiter = ' '
    )]
    include: Option<Vec<i32>>,

    /// Exclude assets in the albums matching the given ids
    #[arg(short = 'x', long = "exclude-albums", group = "ids", num_args = 1.., value_delimiter = ' '
    )]
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
    let args = Arguments::parse();

    let library_path = model::Library::new(PathBuf::from(args.library_path)).db_path();

    run_with_result_handling(|| {
        match args.command {
            Commands::Version => {
                let version = db::with_connection(&library_path, db::get_version_number)?;

                let version_range = db::VersionRange::from_version_number(version)?;
                println!("Library version: {} ({})", version, version_range.description)
            }
            Commands::ListAlbums => {
                let albums = db::with_connection(&library_path, db::get_all_albums)?;
                album_list::print_album_tree(&albums)?
            }
            _ => unimplemented!()
        }

        Ok::<_, Error>(())
    })
}


/// Run the given function and handle any errors that occur.
/// 
/// Errors are saved to a log file and a message is printed to the console.
// TODO Return an exit code the app should return
fn run_with_result_handling<F, R, E>(function: F)
where
    F: Fn() -> std::result::Result<R, E>,
    E: ToString
{
    if let Err(e) = function() {
        let err_log_file = write_error_log(&e.to_string())
            .unwrap_or_else(|e| panic!("Unable to write error log: {}", e));

        eprintln!("{}", "One or more errors occurred executing the given command!".red());
        
        let logfile_msg = format!("For more information, see the error log at '{}'", err_log_file);
        eprintln!("{}", logfile_msg.bright_red());
    }
}

/// Writes the given log string to a file and returns the filename.
fn write_error_log(log: &str) -> std::result::Result<String, String> {
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let filename = format!("apple-photos-export-{}.log", random_suffix);

    let mut report = File::create(&filename)
        .map_err(|e| format!("Unable to create error log: {}", e))?;

    report.write_all(log.as_bytes())
        .map_err(|e| format!("Unable to write to error log: {}", e))?;

    Ok(filename)
}