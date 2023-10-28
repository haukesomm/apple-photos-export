import argparse

from app.service import ExporterService


def main():
    parser = argparse.ArgumentParser(
        prog='photoslibrary-exporter',
        description='Export photos from the macOS Photos app, preserving the original album hierarchy.',
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

    service = ExporterService(parsed_args.library)

    if parsed_args.action == "list-albums":
        service.print_album_tree()
    elif parsed_args.action == "export":
        service.export_assets(parsed_args.destination, parsed_args.restore_original_filenames, parsed_args.dry_run)


if __name__ == '__main__':
    main()
