import argparse

from app.album import service as album_service


def main():
    parser = argparse.ArgumentParser(
        prog='Apple Photos Library Exporter',
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
        help='Path of the destination directory'
    )

    parsed_args = parser.parse_args()

    if parsed_args.action == "list-albums":
        album_service.print_album_tree(parsed_args.library)
    elif parsed_args.action == "export":
        print("Exporting photos is not yet implemented")


if __name__ == '__main__':
    main()
