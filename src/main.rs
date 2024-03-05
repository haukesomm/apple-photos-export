use std::path::Path;

use clap::{Args, Parser, Subcommand};

use crate::album_list::printing::AlbumListPrinter;
use crate::export::copying::{AssetCopyStrategy, DefaultAssetCopyStrategy, DryRunAssetCopyStrategy};
use crate::export::exporter::Exporter;
use crate::export::structure::{AlbumOutputStructureStrategy, JoiningOutputStructureStrategy, OutputStructureStrategy, PlainOutputStructureStrategy, YearMonthOutputStructureStrategy};
use crate::model::library::PhotosLibrary;
use crate::repo::album::AlbumRepositoryImpl;
use crate::repo::asset::{AssetWithAlbumInfoRepoImpl, FilterMode};

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

    /// Only include albums matching the given ids
    #[arg(short = 'i', long = "include", group = "ids", num_args = 1.., value_delimiter = ' ')]
    include: Option<Vec<i32>>,

    /// Exclude all albums matching the given ids
    #[arg(short = 'e', long = "exclude", group = "ids", num_args = 1.., value_delimiter = ' ')]
    exclude: Option<Vec<i32>>,

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

    let library = PhotosLibrary::new(&args.library_path);

    match args.command {
        Commands::ListAlbums => list_albums(&library.db_path()),
        Commands::ExportAssets(export_args) => export_assets(library, export_args)
    }
}

fn list_albums(db_path: &String) {
    let album_repo = AlbumRepositoryImpl::new(&db_path);
    let album_lister = AlbumListPrinter::new(&album_repo);
    album_lister.print_album_tree();
}

fn export_assets(photos_library: PhotosLibrary, args: ExportArgs) {
    let db_path = photos_library.db_path();
    let asset_filter_mode = match (args.include, args.exclude) {
        (Some(ids), _) => FilterMode::IncludeAlbumIds(ids),
        (_, Some(ids)) => FilterMode::ExcludeAlbumIds(ids),
        _ => FilterMode::None
    };
    let repo = AssetWithAlbumInfoRepoImpl::new(&db_path, asset_filter_mode);

    let output_strategy: Box<dyn OutputStructureStrategy> =
        match (args.plain, args.album, args.year_month, args.year_month_album) {
            (true, _, _, _) => Box::new(PlainOutputStructureStrategy::new()),
            (_, true, _, _) => Box::new(AlbumOutputStructureStrategy::new(args.flatten_albums)),
            (_, _, true, _) => Box::new(YearMonthOutputStructureStrategy::asset_date_based()),
            (_, _, _, true) => Box::new(
                JoiningOutputStructureStrategy::new(
                    vec![
                        Box::new(YearMonthOutputStructureStrategy::album_date_based()),
                        Box::new(AlbumOutputStructureStrategy::new(args.flatten_albums))
                    ]
                )
            ),
            _ => Box::new(PlainOutputStructureStrategy::new())
        };

    let copy_strategy: Box<dyn AssetCopyStrategy> = match args.dry_run {
        true => Box::new(DryRunAssetCopyStrategy::new()),
        false => Box::new(DefaultAssetCopyStrategy::new())
    };

    let exporter = Exporter::new(&repo, output_strategy.as_ref(), copy_strategy.as_ref());

    exporter.export(
        Path::new(&photos_library.original_assets_path()),
        Path::new(&args.destination_path),
        args.restore_original_filenames
    );
}