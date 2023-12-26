import argparse

from apple_photos_export import album_tree, library_file
from apple_photos_export.export import exporter
from apple_photos_export.export.properties import ExportProperties
from apple_photos_export.export.strategy import PlainExportStrategy, YearMonthExportStrategy, \
    AlbumExportStrategy, JoiningExportStrategy


def _print_album_tree(parsed_args: argparse.Namespace) -> None:
    album_tree.print_album_tree(library_file.db_path(parsed_args.library))


def _export(parsed_args: argparse.Namespace) -> None:
    properties = ExportProperties(
        library_path=parsed_args.library,
        destination_path=parsed_args.destination,
        export_strategy=parsed_args.strategy(parsed_args.flatten_albums) or PlainExportStrategy(),
        restore_original_filenames=parsed_args.restore_original_filenames,
        dry_run=parsed_args.dry_run,
        flatten_albums=parsed_args.flatten_albums,
        excluded_album_ids=parsed_args.exclude_albums or [],
    )
    exporter.export_assets(properties)


def run_cli():
    """
    Runs the command line interface.
    """
    parser = argparse.ArgumentParser(
        prog='apple-photos-export',
        description='Export photos from the macOS Photos library, organized by album and/or date.',
        add_help=True
    )

    action_parsers = parser.add_subparsers(dest='action', required=True, help='Action to perform')

    list_albums_parser = action_parsers.add_parser("list-albums", help="List all albums")
    list_albums_parser.set_defaults(func=_print_album_tree)
    list_albums_parser.add_argument(
        'library',
        help='Path of the library file'
    )

    export_parser = action_parsers.add_parser("export", help="Export photos")
    export_parser.set_defaults(func=_export)
    export_parser.add_argument(
        'library',
        help='Path of the library file'
    )
    export_parser.add_argument(
        'destination',
        help='path of the destination directory'
    )
    export_parser_strategy = export_parser.add_mutually_exclusive_group()
    export_parser_strategy.add_argument(
        '-p', '--plain',
        help='export photos to the root of the export directory',
        action='store_const',
        dest='strategy',
        const=lambda _: PlainExportStrategy(),
    )
    export_parser_strategy.add_argument(
        '-a', '--album',
        help='export photos grouped by album',
        action='store_const',
        dest='strategy',
        const=lambda flatten_albums: AlbumExportStrategy(flatten_albums),
    )
    export_parser_strategy.add_argument(
        '-y', '--year-month',
        help='export photos grouped by year/month',
        action='store_const',
        dest='strategy',
        const=lambda _: YearMonthExportStrategy(),
    )
    export_parser_strategy.add_argument(
        '-m', '--year-month-album',
        help='export photos grouped by year/month/album',
        action='store_const',
        dest='strategy',
        const=lambda flatten_albums: JoiningExportStrategy(
            YearMonthExportStrategy(YearMonthExportStrategy.album_date_selector),
            AlbumExportStrategy(flatten_albums)
        )
    )
    export_parser.add_argument(
        '-o', '--restore-original-filenames',
        action='store_true',
        dest='restore_original_filenames',
        help='restore the original filenames of the photos',
    )
    export_parser.add_argument(
        '-f', '--flatten-albums',
        help='flatten the album hierarchy',
        action='store_true',
    )
    export_parser.add_argument(
        '-e', '--exclude-albums',
        help='exclude the specified album ids from the export',
        nargs='+',
        type=str,
        dest='exclude_albums',
    )
    export_parser.add_argument(
        '-d', '--dry-run',
        action='store_true',
        dest='dry_run',
        help='do not actually export the photos',
    )

    parsed_args = parser.parse_args()
    parsed_args.func(parsed_args)
