import datetime
from typing import List, Optional

from colors import color
from treelib import Tree

from apple_photos_export.model.album import Album, AlbumKind
from apple_photos_export.repository.albums import get_albums
from apple_photos_export.repository.assets import get_album_asset_counts


def print_album_tree(db_file_path: str) -> None:
    """
    Gets all albums from the library and prints them as an ASCII tree.
    """
    albums = get_albums(db_file_path)
    tree = _generate_ascii_album_tree(albums)
    print(tree)

    asset_counts = get_album_asset_counts(db_file_path)
    print(f'Total number of assets: {asset_counts.total}')
    print(f'Number of assets not in an album: {asset_counts.total - asset_counts.album}')


def _generate_ascii_album_tree(albums: List[Album]):
    tree = Tree()

    albums_by_parent_id = {}
    for album in albums:
        albums_by_parent_id.setdefault(album.parent_album, []).append(album)

    def _add_nodes_recursively(parent_id: Optional[int] = None) -> None:
        """
        Adds all albums as nodes to the tree, recursively, starting with the root album (parent_id=None).
        """
        children = albums_by_parent_id.setdefault(parent_id, [])
        for child in children:
            tree.create_node(identifier=child.id, parent=parent_id, tag=_album_to_str(child), data=child)
            _add_nodes_recursively(child.id)

    _add_nodes_recursively()

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
            color(album.name, fg='silver'),
            color(f' ({album.asset_count} assets)', fg='teal')
        ])

    return description

