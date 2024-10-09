use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use db::version::check_library_version;

use crate::album_list::print_album_tree;
use crate::changelog::print_changelog;
use crate::db::repo::album::AlbumRepository;
use crate::db::repo::asset::{AlbumFilter, AssetRepository, HiddenAssetsFilter};
use crate::export::copying::{AbsolutePathBuildingCopyOperationFactoryDecorator, AssetCopyStrategy, CombiningCopyOperationFactory, CopyOperationFactory, DefaultAssetCopyStrategy, DerivatesCopyOperationFactory, DryRunAssetCopyStrategy, FilenameRestoringCopyOperationFactoryDecorator, OriginalsCopyOperationFactory, OutputStructureCopyOperationFactoryDecorator, SuffixSettingCopyOperationFactoryDecorator};
use crate::export::export_assets;
use crate::export::structure::{AlbumOutputStrategy, HiddenAssetHandlingOutputStrategyDecorator, NestingOutputStrategyDecorator, OutputStrategy, PlainOutputStrategy, YearMonthOutputStrategy};
use crate::result::PhotosExportResult;

mod album_list;
mod export;
mod util;
mod changelog;
mod db;
mod foundation;
mod model;
mod result;


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
    let args = Arguments::parse();

    let result: PhotosExportResult<()> = match args.command {
        Commands::Changelog => print_changelog(),
        Commands::ListAlbums(list_args) => {
            let database_path = get_database_path(&list_args.library_path);

            check_library_version(&database_path)
                .and_then(|_| {
                    print_album_tree(
                        get_database_path(&list_args.library_path)
                    )
                })
        },
        Commands::Export(export_args) => {
            let database_path = get_database_path(&export_args.library_path);

            check_library_version(&database_path)
                .and_then(|_| run_photos_export(&export_args))
        },
    };

    // Handle uncaught errors and print them to stderr
    // Errors requiring more complex handling may have already been handled at this point
    if let Err(e) = result {
        for message in &e.messages {
            eprintln!("{} {}", "Error:".red(), message);
        }
        std::process::exit(1);
    }
}


fn get_database_path(library_path: &str) -> String {
    PathBuf::new()
        .join(library_path)
        .join("database")
        .join("Photos.sqlite")
        .to_string_lossy()
        .to_string()
}


fn run_photos_export(export_args: &ExportArgs) -> PhotosExportResult<()> {
    let db_path = get_database_path(&export_args.library_path);

    let asset_repo = setup_asset_repo(db_path.clone(), export_args);
    let copy_operation_factory = setup_copy_operation_factory(db_path.clone(), export_args)?;
    let copy_strategy = setup_copy_strategy(export_args.dry_run);

    export_assets(asset_repo, copy_operation_factory, copy_strategy)
}

fn setup_asset_repo(db_path: String, args: &ExportArgs) -> AssetRepository {
    let hidden_asset_filter = if args.include_hidden {
        HiddenAssetsFilter::Include
    } else if args.must_be_hidden {
        HiddenAssetsFilter::Only
    } else {
        HiddenAssetsFilter::Exclude
    };

    let album_filter = if let Some(ids) = args.include.clone() {
        AlbumFilter::Include(ids)
    } else if let Some(ids) = args.exclude.clone() {
        AlbumFilter::Exclude(ids)
    } else {
        AlbumFilter::None
    };

    AssetRepository::new(db_path, hidden_asset_filter, album_filter)
}

fn setup_copy_operation_factory(
    db_path: String,
    args: &ExportArgs
) -> PhotosExportResult<Box<dyn CopyOperationFactory>> {
    let factory: Box<dyn CopyOperationFactory> = Box::new(
        AbsolutePathBuildingCopyOperationFactoryDecorator::new(
            PathBuf::from(&args.library_path),
            PathBuf::from(&args.output_dir),
            Box::new(
                OutputStructureCopyOperationFactoryDecorator::new(
                    if args.include_edited {
                        Box::new(
                            CombiningCopyOperationFactory::new(
                                vec![
                                    Box::new(
                                        SuffixSettingCopyOperationFactoryDecorator::new(
                                            Box::new(OriginalsCopyOperationFactory::new()),
                                            "_original".to_string()
                                        )
                                    ),
                                    Box::new(DerivatesCopyOperationFactory::new())
                                ]
                            )
                        )
                    } else if args.only_edited {
                        Box::new(DerivatesCopyOperationFactory::new())
                    } else {
                        Box::new(OriginalsCopyOperationFactory::new())
                    },
                    setup_output_strategy(db_path, args)?
                )
            )
        )
    );

    Ok(
        if args.restore_original_filenames {
            Box::new(
                FilenameRestoringCopyOperationFactoryDecorator::new(factory)
            )
        } else {
            factory
        }
    )
}

fn setup_output_strategy(
    db_path: String,
    args: &ExportArgs
) -> PhotosExportResult<Box<dyn OutputStrategy>> {

    let strategy: Box<dyn OutputStrategy> = if args.album {
        Box::new(
            AlbumOutputStrategy::new(
                args.flatten_albums,
                AlbumRepository::new(db_path).get_all()?
            )
        )
    } else if args.year_month {
        Box::new(YearMonthOutputStrategy::asset_date_based())
    } else if args.year_month_album {
        Box::new(
            NestingOutputStrategyDecorator::new(
                vec![
                    Box::new(YearMonthOutputStrategy::album_date_based()),
                    Box::new(
                        AlbumOutputStrategy::new(
                            args.flatten_albums,
                            AlbumRepository::new(db_path).get_all()?
                        )
                    )
                ]
            )
        )
    } else {
        Box::new(PlainOutputStrategy::new())
    };

    Ok(
        Box::new(
            HiddenAssetHandlingOutputStrategyDecorator::new(strategy)
        )
    )
}

fn setup_copy_strategy(dry_run: bool) -> Box<dyn AssetCopyStrategy> {
    if dry_run {
        Box::new(DryRunAssetCopyStrategy::new())
    } else {
        Box::new(DefaultAssetCopyStrategy::new())
    }
}