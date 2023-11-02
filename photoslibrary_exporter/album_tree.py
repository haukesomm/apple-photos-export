import datetime
from typing import List

from colors import color
from treelib import Tree

from photoslibrary_exporter.model import Album, AlbumKind


def generate_ascii_album_tree(albums: List[Album]):
    tree = Tree()

    root = next(album for album in albums if album.kind == AlbumKind.ROOT)
    tree.create_node(identifier=root.id, parent=None, tag=_album_to_str(root), data=root)

    _add_child_nodes(tree=tree, albums=albums, parent_id=root.id)

    return tree.show(stdout=False, key=lambda a: a.data.start_date or datetime.datetime.min)


def _add_child_nodes(tree: Tree, albums: List[Album], parent_id: str) -> None:
    children = [album for album in albums if album.parent_album == parent_id]
    for child in children:
        tree.create_node(identifier=child.id, parent=parent_id, tag=_album_to_str(child), data=child)
        _add_child_nodes(tree, albums, child.id)


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
