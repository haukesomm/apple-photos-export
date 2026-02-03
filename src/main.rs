use crate::export::task::mapping::mappers;
use crate::export::{ExportEngine, ExportMetadata};
use crate::model::Library;
use crate::result::{Error, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

mod album_list;
mod cocoa_time;
mod confirmation;
mod db;
mod export;
pub mod fs;
mod model;
mod result;
mod uti;

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
    ///
    /// Note: This option only has an effect when using an album-based grouping strategy!
    #[arg(
        short = 'a', 
        long = "include-by-album", 
        group = "ids", 
        num_args = 1.., 
        value_delimiter = ','
    )]
    include_by_album: Option<Vec<i32>>,

    /// Exclude assets in the albums matching the given ids
    ///
    /// Note: This option only has an effect when using an album-based grouping strategy!
    #[arg(
        short = 'A', 
        long = "exclude-by-album", 
        group = "ids", 
        num_args = 1.., 
        value_delimiter = ',')
    ]
    exclude_by_album: Option<Vec<i32>>,

    /// Only include assets that are not part of the 'hidden' album
    #[arg(short = 'v', long = "visible")]
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

    /// Prefer the edited version of the asset if available and fall back to the original otherwise
    #[arg(short = 'E', long = "prefer-edited", group = "edited")]
    prefer_edited: bool,

    /// Don't copy files that already exist in the output directory.
    #[arg(short = 's', long = "skip")]
    skip_existing: bool,

    /// Delete files in the output directory that are not part of the current export.
    #[arg(long = "delete")]
    delete: bool,

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
                let version = db::with_connection(&db_path, db::version::get_version)?;
                println!("Library version: {}", version)
            }
            Commands::ListAlbums => {
                let albums = db::with_connection(&db_path, |conn| {
                    db::version::perform_version_check(conn)?;
                    db::album::get_all_albums(conn)
                })?;
                album_list::print_album_tree(&albums)?
            }
            Commands::Export(export_args) => {
                let (albums, asset_count, exportable_assets) =
                    db::with_connection(&db_path, |conn| {
                        db::version::perform_version_check(conn)?;

                        Ok((
                            db::album::get_all_albums(conn)?
                                .into_iter()
                                .map(|album| (album.id, album))
                                .collect(),
                            db::asset::get_visible_count(conn)?,
                            db::asset::get_exportable_assets(conn)?,
                        ))
                    })?;

                let exportable_asset_count = exportable_assets.len();

                let mut builder = {
                    use export::factory::ExportTaskFactory;
                    if export_args.include_edited {
                        ExportTaskFactory::new_for_originals_and_derivates(library.clone())
                    } else if export_args.prefer_edited {
                        ExportTaskFactory::new_for_derivates_with_fallback(library.clone())
                    } else {
                        ExportTaskFactory::new_for_originals(library.clone())
                    }
                };

                if export_args.restore_original_filenames {
                    builder.add_mapper(mappers::RestoreOriginalFilenames)
                }

                if export_args.include_edited {
                    builder.add_mapper(mappers::MarkOriginalsAndDerivates)
                }

                if export_args.album || export_args.year_month_album {
                    builder.add_mapper(mappers::OneTaskPerAlbum);

                    if export_args.flatten_albums {
                        builder.add_mapper(mappers::GroupByAlbum::flat(&albums))
                    } else {
                        builder.add_mapper(mappers::GroupByAlbum::recursive(&albums))
                    }
                }

                if export_args.year_month_album {
                    builder.add_mapper(mappers::GroupByYearMonthAndAlbum::new(&albums))
                }

                if export_args.year_month {
                    builder.add_mapper(mappers::GroupByYearAndMonth)
                }

                if let Some(ids) = &export_args.include_by_album {
                    builder.add_mapper(mappers::FilterByAlbumId::new(
                        ids.clone(),
                        mappers::AlbumFilterMode::Include,
                    ));
                }

                if let Some(ids) = &export_args.exclude_by_album {
                    builder.add_mapper(mappers::FilterByAlbumId::new(
                        ids.clone(),
                        mappers::AlbumFilterMode::Exclude,
                    ));
                }

                if export_args.visible {
                    builder.add_mapper(mappers::ExcludeHidden)
                } else {
                    builder.add_mapper(mappers::PrefixHidden)
                }

                builder.add_mapper(mappers::ConvertToAbsolutePath::new(&export_args.output_dir));

                // Keep track of existing files in the output directory
                //
                // This is used for the 'skip existing' and 'delete' options:
                // - For 'skip existing', we need to know which files already exist so we can skip them
                // - For 'delete', we need to know which files have not occurred in the export so we can delete them
                //
                // In order to do so, we first gather all existing files in the output directory
                // before we start building the export tasks. Then, when building the export tasks,
                // we mark files that are going to be exported as 'handled' via the `SkipIfExists` mapper.
                // Finally, after building the export tasks, we can determine which files are unhandled
                // and create delete tasks for them.
                let existing_unhandled_output_files: Rc<RefCell<HashSet<PathBuf>>> =
                    Rc::new(RefCell::new(HashSet::new()));

                if export_args.skip_existing || export_args.delete {
                    println!(
                        "Indexing existing files in output directory (this may take a long time) ..."
                    );
                    existing_unhandled_output_files
                        .borrow_mut()
                        .extend(fs::recursively_get_files(&export_args.output_dir));

                    if export_args.skip_existing {
                        builder.add_mapper(mappers::SkipIfExists::new(Rc::clone(
                            &existing_unhandled_output_files,
                        )));
                    }

                    builder.add_mapper(mappers::RemoveFromCacheIfExists::new(Rc::clone(
                        &existing_unhandled_output_files,
                    )));
                }

                let mut export_tasks = builder.build(exportable_assets);

                if export_args.delete {
                    let borrow = existing_unhandled_output_files.borrow();
                    let delete_tasks = export::task::create_delete_tasks(borrow.iter());
                    export_tasks.extend(delete_tasks);
                }

                let engine: ExportEngine = if export_args.dry_run {
                    ExportEngine::dry_run()
                } else {
                    ExportEngine::new()
                };

                let export_metadata = ExportMetadata {
                    total_asset_count: asset_count,
                    exportable_asset_count,
                    export_task_count: export_tasks.len(),
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
fn run_with_result_handling<F, R>(function: F)
where
    F: Fn() -> std::result::Result<R, Error>,
{
    if let Err(e) = function() {
        match e {
            Error::General(msg) => {
                eprintln!("{}", msg.bright_red());
            }
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
                eprintln!(
                    "{}",
                    "One or more errors occurred during the export!".bright_red()
                );

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
fn _write_export_error_log(log: &Vec<String>) -> std::result::Result<String, String> {
    let random_suffix: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let filename = format!("apple-photos-export-{}.log", random_suffix);

    let mut report =
        File::create(&filename).map_err(|e| format!("Unable to create error log: {}", e))?;

    for error in log {
        report
            .write(error.as_bytes())
            .map_err(|e| format!("Unable to write to error log: {}", e))?;
    }

    Ok(filename)
}
