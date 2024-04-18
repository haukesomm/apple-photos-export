use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::album_list::print_album_tree;
use crate::changelog::print_changelog;
use crate::db::repo::album::AlbumRepository;
use crate::db::repo::asset::{AlbumFilter, AssetRepository, HiddenAssets};
use crate::export::copying::{AbsolutePathBuildingCopyOperationFactoryDecorator, AssetCopyStrategy, CombiningCopyOperationFactory, CopyOperationFactory, DefaultAssetCopyStrategy, DerivatesCopyOperationFactory, DryRunAssetCopyStrategy, FilenameRestoringCopyOperationFactoryDecorator, OriginalsCopyOperationFactory, OutputStructureCopyOperationFactoryDecorator, SuffixSettingCopyOperationFactoryDecorator};
use crate::export::exporter::Exporter;
use crate::export::structure::{AlbumOutputStrategy, HiddenAssetHandlingOutputStrategyDecorator, NestingOutputStrategyDecorator, OutputStrategy, PlainOutputStrategy, YearMonthOutputStrategy};
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

    match args.command {
        Commands::Changelog => print_changelog().unwrap(),
        Commands::ListAlbums(list_args) => print_album_tree(list_args.library_path),
        Commands::Export(export_args) => export_assets(export_args)
    }
}


fn export_assets(args: ExportArgs) {
    let library_path = args.library_path.clone();
    let photos_library = PhotosLibrary::new(library_path.clone());

    let asset_repo = setup_asset_repo(photos_library.db_path(), &args);
    let copy_operation_factory = setup_copy_operation_factory(photos_library.db_path(), &args);
    let copy_strategy = setup_copy_strategy(args.dry_run);

    let exporter = Exporter::new(
        asset_repo,
        copy_operation_factory,
        copy_strategy,
    );

    let result = exporter.export();

    if let Err(e) = result {
        eprintln!("Unexpected error during the asset export: {}", e);
    }
}

fn setup_asset_repo(db_path: String, args: &ExportArgs) -> AssetRepository {
    let hidden_asset_filter = if args.include_hidden {
        HiddenAssets::Include
    } else if args.must_be_hidden {
        HiddenAssets::Require
    } else {
        HiddenAssets::Exclude
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

fn setup_copy_operation_factory(db_path: String, args: &ExportArgs) -> Box<dyn CopyOperationFactory> {
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
                    setup_output_strategy(db_path, args)
                )
            )
        )
    );

    if args.restore_original_filenames {
        Box::new(
            FilenameRestoringCopyOperationFactoryDecorator::new(factory)
        )
    } else {
        factory
    }
}

fn setup_output_strategy(db_path: String, args: &ExportArgs) -> Box<dyn OutputStrategy> {

    // TODO: Clean up this function and use result
    fn setup_album_output_strategy(db_path: String, flatten_albums: bool) -> Box<dyn OutputStrategy> {
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
            AlbumOutputStrategy::new(
                flatten_albums,
                albums_by_id
            )
        )
    }

    let strategy: Box<dyn OutputStrategy> = if args.album {
        setup_album_output_strategy(db_path, args.flatten_albums)
    } else if args.year_month {
        Box::new(YearMonthOutputStrategy::asset_date_based())
    } else if args.year_month_album {
        Box::new(
            NestingOutputStrategyDecorator::new(
                vec![
                    Box::new(YearMonthOutputStrategy::album_date_based()),
                    setup_album_output_strategy(db_path, args.flatten_albums)
                ]
            )
        )
    } else {
        Box::new(PlainOutputStrategy::new())
    };

    Box::new(
        HiddenAssetHandlingOutputStrategyDecorator::new(strategy)
    )
}

fn setup_copy_strategy(dry_run: bool) -> Box<dyn AssetCopyStrategy> {
    if dry_run {
        Box::new(DryRunAssetCopyStrategy::new())
    } else {
        Box::new(DefaultAssetCopyStrategy::new())
    }
}