import datetime
from typing import List

from colors import color
from treelib import Tree

import photoslibrary_exporter.repository.albums
import photoslibrary_exporter.repository.assets
from photoslibrary_exporter import cocoa
from photoslibrary_exporter.model.album import Album, AlbumKind
from photoslibrary_exporter.repository.albums import AlbumDto


def print_album_tree(db_file_path: str) -> None:
    """
    Gets all albums from the library and prints them as an ASCII tree.
    """

    albums = _get_albums(db_file_path)
    tree = _generate_ascii_album_tree(albums)
    print(tree)

    asset_counts = photoslibrary_exporter.repository.assets.get_album_asset_counts(db_file_path)
    print(f'Total number of assets: {asset_counts.asset_count}')
    print(f'Number of assets not in an album: {asset_counts.asset_count_no_album}')


def _get_albums(library_path: str) -> list[Album]:
    """
    Gets all albums from the library and returns them as a list of Album objects.
    """

    def parse_dto(dto: AlbumDto) -> Album:
        return Album(
            id=dto.id,
            kind=AlbumKind(dto.kind),
            parent_album=dto.parent_album,
            name=dto.name,
            start_date=cocoa.cocoa_timestamp_to_datetime(dto.cocoa_start_date) if dto.cocoa_start_date else None
        )

    album_dtos = photoslibrary_exporter.repository.albums.get_albums(library_path)
    return list(map(parse_dto, album_dtos))


def _generate_ascii_album_tree(albums: List[Album]):
    tree = Tree()

    root = next(album for album in albums if album.kind == AlbumKind.ROOT)
    tree.create_node(identifier=root.id, parent=None, tag=_album_to_str(root), data=root)

    def _add_child_nodes(parent_id: str) -> None:
        children = [album for album in albums if album.parent_album == parent_id]
        for child in children:
            tree.create_node(identifier=child.id, parent=parent_id, tag=_album_to_str(child), data=child)
            _add_child_nodes(child.id)

    _add_child_nodes(parent_id=root.id)

    return tree.show(stdout=False, key=lambda a: a.data.start_date or datetime.datetime.min)


def _album_to_str(album: Album) -> str:
    if album.kind == AlbumKind.ROOT:
        description = color('<root album>', bg='purple', fg='white')
    elif album.kind == AlbumKind.USER_FOLDER:
        description = color(album.name, bg='navy', fg='white')
    else:
        timestamp = album.start_date.strftime("%Y-%m-%d %H:%M:%S") if album.start_date else None
        description = ''.join([
            color(f'({album.id}) ', fg='yellow'),
            color(f'{timestamp}: ', fg='gray') if timestamp else '',
            color(album.name, fg='silver')
        ])

    return description

