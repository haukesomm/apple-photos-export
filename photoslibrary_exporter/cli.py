import argparse

from photoslibrary_exporter import album_list, asset_export, library_file


def run_cli():
    """
    Runs the command line interface.
    """

    parser = argparse.ArgumentParser(
        prog='photoslibrary-exporter',
        description='Export photos from the macOS Photos photoslibrary_exporter, preserving the original album hierarchy.',
        add_help=True
    )

    action_parsers = parser.add_subparsers(dest='action', required=True, help='Action to perform')

    list_albums_parser = action_parsers.add_parser("list-albums", help="List all albums")
    list_albums_parser.add_argument(
        'library',
        help='Path of the library file'
    )

    export_parser = action_parsers.add_parser("export", help="Export photos")
    export_parser.add_argument(
        'library',
        help='Path of the library file'
    )
    export_parser.add_argument(
        'destination',
        help='path of the destination directory'
    )
    export_parser.add_argument(
        '-r', '--restore-original-filenames',
        action='store_true',
        dest='restore_original_filenames',
        help='restore the original filenames of the photos',
    )
    export_parser.add_argument(
        '-d', '--dry-run',
        action='store_true',
        dest='dry_run',
        help='do not actually export the photos',
    )

    parsed_args = parser.parse_args()

    library_file_path = parsed_args.library
    database_file_path = library_file.get_photos_db_path(library_file_path)

    if parsed_args.action == "list-albums":
        album_list.print_album_tree(database_file_path)
    elif parsed_args.action == "export":
        asset_export.export_assets(
            library_file_path,
            parsed_args.destination,
            parsed_args.restore_original_filenames,
            parsed_args.dry_run
        )
