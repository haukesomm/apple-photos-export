use std::path::Path;

use clap::{Args, Parser, Subcommand};

use crate::album_list::printing::AlbumListPrinter;
use crate::export::copying::{AssetCopyStrategy, DefaultAssetCopyStrategy, DryRunAssetCopyStrategy};
use crate::export::exporter::Exporter;
use crate::export::structure::{AlbumOutputStructureStrategy, JoiningOutputStructureStrategy, OutputStructureStrategy, PlainOutputStructureStrategy, YearMonthOutputStructureStrategy};
use crate::model::library::PhotosLibrary;
use crate::repo::album::AlbumRepository;
use crate::repo::asset::{AssetWithAlbumInfoRepo, FilterMode};

mod model;
mod album_list;
mod repo;
mod export;
mod confirmation;

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

    /// Output directory
    output_dir: String,

    /// Output by album
    #[arg(short = 'a', long = "by-album", group = "strategy")]
    album: bool,

    /// Output by year/month
    #[arg(short = 'm', long = "by-year-month", group = "strategy")]
    year_month: bool,

    /// Output by year/month/album
    #[arg(short = 'M', long = "by-year-month-album", group = "strategy")]
    year_month_album: bool,

    /// Include albums matching the given ids
    #[arg(short = 'i', long = "include", group = "ids", num_args = 1.., value_delimiter = ' ')]
    include: Option<Vec<i32>>,

    /// Exclude albums matching the given ids
    #[arg(short = 'e', long = "exclude", group = "ids", num_args = 1.., value_delimiter = ' ')]
    exclude: Option<Vec<i32>>,

    /// Restore original filenames
    #[arg(short = 'r', long = "restore-original-filenames")]
    restore_original_filenames: bool,

    /// Flatten album structure
    #[arg(short = 'f', long = "flatten-albums")]
    flatten_albums: bool,

    /// Dry run
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,
}

fn main() {
    let args = Arguments::parse();

    let library = PhotosLibrary::new(&args.library_path);

    match args.command {
        Commands::ListAlbums => list_albums(library.db_path()),
        Commands::ExportAssets(export_args) => export_assets(library, export_args)
    }
}

fn list_albums(db_path: String) {
    let album_repo = AlbumRepository::new(db_path);
    let album_lister = AlbumListPrinter::new(&album_repo);
    album_lister.print_album_tree();
}

fn export_assets(photos_library: PhotosLibrary, args: ExportArgs) {
    let asset_repo = {
        let filter = if let Some(ids) = args.include {
            FilterMode::IncludeAlbumIds(ids)
        } else if let Some(ids) = args.exclude {
            FilterMode::ExcludeAlbumIds(ids)
        } else {
            FilterMode::None
        };

        AssetWithAlbumInfoRepo::new(photos_library.db_path(), filter)
    };

    let output_strategy: Box<dyn OutputStructureStrategy> = if args.album {
        Box::new(AlbumOutputStructureStrategy::new(args.flatten_albums))
    } else if args.year_month {
        Box::new(YearMonthOutputStructureStrategy::asset_date_based())
    } else if args.year_month_album {
        Box::new(
            JoiningOutputStructureStrategy::new(
                vec![
                    Box::new(YearMonthOutputStructureStrategy::album_date_based()),
                    Box::new(AlbumOutputStructureStrategy::new(args.flatten_albums))
                ]
            )
        )
    } else {
        Box::new(PlainOutputStructureStrategy::new())
    };

    let copy_strategy: Box<dyn AssetCopyStrategy> = if args.dry_run {
        Box::new(DryRunAssetCopyStrategy::new())
    } else {
        Box::new(DefaultAssetCopyStrategy::new())
    };

    let exporter = Exporter::new(&asset_repo, output_strategy.as_ref(), copy_strategy.as_ref());
    exporter.export(
        Path::new(&photos_library.original_assets_path()),
        Path::new(&args.output_dir),
        args.restore_original_filenames
    );
}