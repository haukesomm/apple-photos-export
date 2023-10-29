from photoslibrary_exporter import cocoa, repo as album_repo, album_tree
from photoslibrary_exporter.model import Album, AlbumKind
from photoslibrary_exporter.repo import AlbumDto


def print_album_tree(db_file_path: str) -> None:
    """
    Gets all albums from the library and prints them as an ASCII tree.
    """

    albums = get_albums(db_file_path)
    tree = album_tree.generate_ascii_album_tree(albums)
    print(tree)

    asset_counts = album_repo.get_album_asset_counts(db_file_path)
    print(f'Total number of assets: {asset_counts.asset_count}')
    print(f'Number of assets not in an album: {asset_counts.asset_count_no_album}')


def get_albums(library_path: str) -> list[Album]:
    """
    Gets all albums from the library and returns them as a list of Album objects.
    """

    album_dtos = album_repo.get_albums(library_path)
    return list(map(_parse_album_dto, album_dtos))


def _parse_album_dto(dto: AlbumDto) -> Album:
    return Album(
        id=dto.id,
        kind=AlbumKind(dto.kind),
        parent_album=dto.parent_album,
        name=dto.name,
        start_date=cocoa.cocoa_timestamp_to_datetime(dto.cocoa_start_date) if dto.cocoa_start_date else None
    )
