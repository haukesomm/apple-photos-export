use crate::model::Library;
use crate::result::{Error, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use crate::export::{ExportEngine, ExportMetadata};
use crate::export::modifiers::AlbumFilterMode;

mod db;
mod foundation;
mod model;
mod result;
mod album_list;
mod export;
mod confirmation;

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
    LibraryVersion,

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
    #[arg(short = 'l', long = "group-by-album", group = "strategy")]
    album: bool,

    /// Group assets by year/month
    #[arg(short = 'm', long = "group-by-year-month", group = "strategy")]
    year_month: bool,

    /// Group assets by year/month/album
    #[arg(short = 'M', long = "group-by-year-month-album", group = "strategy")]
    year_month_album: bool,

    /// Include assets in the albums matching the given ids
    #[arg(short = 'a', long = "include-by-album", group = "ids", num_args = 1.., 
    value_delimiter = ',')]
    include_by_album: Option<Vec<i32>>,

    /// Exclude assets in the albums matching the given ids
    #[arg(short = 'A', long = "exclude-by-album", group = "ids", num_args = 1.., 
    value_delimiter = ',')]
    exclude_by_album: Option<Vec<i32>>,

    /// Only include assets that are not part of the 'hidden' album
    #[arg(short = 'v', long = "visible", group = "hidden")]
    visible: bool,

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

    let library = Library::new(PathBuf::from(&args.library_path));
    let db_path = library.db_path();

    run_with_result_handling(|| {
        match &args.command {
            Commands::LibraryVersion => {
                let version = db::with_connection(&db_path, db::get_version_number)?;

                let version_range = db::VersionRange::from_version_number(version)?;
                println!("Library version: {} ({})", version, version_range.description)
            }
            Commands::ListAlbums => {
                let albums = db::with_connection(&db_path, |conn| {
                    use db::*;
                    
                    perform_version_check(conn)?;
                    
                    get_all_albums(conn)
                })?;
                album_list::print_album_tree(&albums)?
            }
            Commands::Export(export_args) => {
                let (albums, asset_count, exportable_assets) = db::with_connection(&db_path, |conn| {
                    use db::*;
                    
                    perform_version_check(conn)?;
                    
                    Ok((
                       get_all_albums(conn)?
                           .into_iter()
                           .map(|album| (album.id, album))
                           .collect(),
                        get_visible_count(conn)?,
                        get_exportable_assets(conn)?
                    ))
                })?;
                
                
                let config = export::builder::TasksBuilderConfig::new(
                    &library,
                    &exportable_assets,
                    &albums,
                    &export_args.output_dir,
                );
                
                let mut builder = {
                    use export::builder::TasksBuilder;
                    if export_args.include_edited {
                        TasksBuilder::for_originals_and_derivates(config)
                    } else if export_args.only_edited {
                        TasksBuilder::for_derivates(config)
                    } else {
                        TasksBuilder::for_originals(config)
                    }
                };
                
                if export_args.restore_original_filenames {
                    builder.add_modifier(export::modifiers::restore_original_filename)
                }
                
                if export_args.include_edited {
                    builder.add_modifier(export::modifiers::mark_originals_and_derivates);
                }

                
                if export_args.album || export_args.year_month_album {
                    builder.create_per_album_tasks();
                    builder.add_modifier(
                        if export_args.flatten_albums {
                            export::modifiers::structure_by_album
                        } else {
                            export::modifiers::structure_by_album_recursively
                        }
                    );
                }

                if export_args.year_month_album {
                    builder.add_modifier(export::modifiers::prefix_with_album_year_and_month)
                }
                
                if export_args.year_month {
                    builder.add_modifier(export::modifiers::prefix_with_asset_year_and_month)
                }
                
                if let Some(ids) = &export_args.include_by_album {
                    builder.add_modifier(
                        export::modifiers::create_album_filtering_modifier(
                            ids.clone(),
                            AlbumFilterMode::Include
                        )
                    );
                }

                if let Some(ids) = &export_args.exclude_by_album {
                    builder.add_modifier(
                        export::modifiers::create_album_filtering_modifier(
                            ids.clone(),
                            AlbumFilterMode::Exclude
                        )
                    );
                }

                if export_args.visible {
                    builder.add_modifier(export::modifiers::exclude_hidden)
                } else {
                    builder.add_modifier(export::modifiers::prefix_hidden_assets);
                }
                
                let export_tasks = builder.build();
                
                
                let engine = if export_args.dry_run {
                    ExportEngine::dry_run()
                } else {
                    ExportEngine::new()
                };
                
                
                let export_metadata = ExportMetadata {
                    total_asset_count: asset_count,
                    exportable_asset_count: exportable_assets.len(),
                    export_task_count: export_tasks.len()
                };
                
                engine.run_export(export_tasks, export_metadata)?;
            }
        }

        Ok::<_, Error>(())
    })
}


/// Run the given function and handle any errors that occur.
/// 
/// Errors are saved to a log file and a message is printed to the console.
// TODO Return an exit code the app should return
fn run_with_result_handling<F, R>(function: F)
where
    F: Fn() -> std::result::Result<R, Error>,
{
    if let Err(e) = function() {
        match e {
            Error::General(msg) => {
                eprintln!("{}", msg.bright_red());
            },
            Error::Database(err) => {
                eprintln!(
                    "{}", 
                    format!("An error occurred connecting to the database: {}", err).bright_red()
                );
            }
            Error::Export(messages) => {
                let err_log_file = _write_export_error_log(&messages)
                    .unwrap_or_else(|e| panic!("Unable to write error log: {}", e));

                eprintln!();
                eprintln!("{}", "One or more errors occurred during the export!".bright_red());

                let logfile_msg = format!(
                    "For more information, see the error log at '{}'", 
                    err_log_file
                );
                eprintln!("{}", logfile_msg.bright_red());
            }
        }
    }
}

/// Writes the given log string to a file and returns the filename.
fn _write_export_error_log(log: &Vec<(String, String)>) -> std::result::Result<String, String> {
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let filename = format!("apple-photos-export-{}.log", random_suffix);

    let mut report = File::create(&filename)
        .map_err(|e| format!("Unable to create error log: {}", e))?;
    
    for (asset_name, error) in log {
        report
            .write(format!("{}: {}\n", asset_name, error).as_bytes())
            .map_err(|e| format!("Unable to write to error log: {}", e))?;
    }

    Ok(filename)
}


/// Performs a version check on the database and returns an error if the version is not
/// supported.
fn perform_version_check(db_conn: &rusqlite::Connection) -> Result<()> {
    use db::*;
    
    let version_number = get_version_number(db_conn)?;
    let version_range = VersionRange::from_version_number(version_number)?;
    let supported = CURRENTLY_SUPPORTED_VERSION;
    
    if version_number < supported.start || version_number > supported.end {
        Err(
            Error::General(
                format!(
                    "Unsupported library version!\nYour version: {} ({})\n\
                    Currently supported version: {} ({} to {})",
                    version_range.description,
                    version_number,
                    supported.description,
                    supported.start,
                    supported.end
                )
            )
        )
    } else {
        Ok(())
    }
}