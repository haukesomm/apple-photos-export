use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::album_list::print_album_tree;
use crate::changelog::print_changelog;
use crate::db::repo::album::AlbumRepository;
use crate::db::repo::exportable_assets::{AlbumFilter, ExportableAssetsRepository};
use crate::export::copying::{AssetCopyStrategy, DefaultAssetCopyStrategy, DryRunAssetCopyStrategy};
use crate::export::exporter::Exporter;
use crate::export::structure::{AlbumOutputStructureStrategy, JoiningOutputStructureStrategy, OutputStructureStrategy, PlainOutputStructureStrategy, YearMonthOutputStructureStrategy};
use crate::library::PhotosLibrary;
use crate::model::FromDbModel;

mod album_list;
mod export;
mod util;
mod changelog;
mod db;
mod foundation;
mod library;
mod model;


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
    #[arg(short = 'e', long = "exclude-albums", group = "ids", num_args = 1.., value_delimiter = ' ')]
    exclude: Option<Vec<i32>>,

    /// Include hidden assets
    #[arg(short = 'H', long = "include-hidden")]
    include_hidden: bool,

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

    match args.command {
        Commands::Changelog => print_changelog().unwrap(),
        Commands::ListAlbums(list_args) => print_album_tree(list_args.library_path),
        Commands::Export(export_args) => export_assets(export_args)
    }
}


fn export_assets(args: ExportArgs) {
    let photos_library = PhotosLibrary::new(args.library_path.clone());

    let asset_repo = setup_asset_repo(photos_library.db_path(), &args);
    let output_strategy = setup_output_strategy(photos_library.db_path(), &args);
    let copy_strategy = setup_copy_strategy(args.dry_run);

    let exporter = Exporter::new(
        asset_repo,
        output_strategy,
        copy_strategy,
        args.restore_original_filenames,
        PathBuf::from(photos_library.original_assets_path()),
        PathBuf::from(args.output_dir)
    );

    let result = exporter.export();

    if let Err(e) = result {
        eprintln!("Unexpected error during the asset export: {}", e);
    }
}

fn setup_asset_repo(db_path: String, args: &ExportArgs) -> ExportableAssetsRepository {
    ExportableAssetsRepository::new(
        db_path,
        args.include_hidden,
        {
            if let Some(ids) = args.include.clone() {
                AlbumFilter::Include(ids)
            } else if let Some(ids) = args.exclude.clone() {
                AlbumFilter::Exclude(ids)
            } else {
                AlbumFilter::None
            }
        }
    )
}

fn setup_output_strategy(db_path: String, args: &ExportArgs) -> Box<dyn OutputStructureStrategy> {

    // TODO: Find a more elegant solution
    fn setup_album_output_strategy(db_path: String, flatten_albums: bool) -> Box<dyn OutputStructureStrategy> {
        let album_repo = AlbumRepository::new(db_path);
        let albums_by_id = album_repo
            .get_all()
            .unwrap()
            .into_iter()
            .map(|a| {
                let album = model::album::Album::from_db_model(a)
                    .expect("Failed to convert Album from DB model");
                (album.id, album)
            })
            .collect();

        Box::new(
            AlbumOutputStructureStrategy::new(
                flatten_albums,
                albums_by_id
            )
        )
    }

    if args.album {
        setup_album_output_strategy(db_path, args.flatten_albums)
    } else if args.year_month {
        Box::new(YearMonthOutputStructureStrategy::asset_date_based())
    } else if args.year_month_album {
        Box::new(
            JoiningOutputStructureStrategy::new(
                vec![
                    Box::new(YearMonthOutputStructureStrategy::album_date_based()),
                    setup_album_output_strategy(db_path, args.flatten_albums)
                ]
            )
        )
    } else {
        Box::new(PlainOutputStructureStrategy::new())
    }
}

fn setup_copy_strategy(dry_run: bool) -> Box<dyn AssetCopyStrategy> {
    if dry_run {
        Box::new(DryRunAssetCopyStrategy::new())
    } else {
        Box::new(DefaultAssetCopyStrategy::new())
    }
}