from typing import List

from treelib import Tree

from photoslibrary_exporter.model import Album, AlbumKind


def generate_ascii_album_tree(albums: List[Album]):
    tree = Tree()

    root = next(album for album in albums if album.kind == AlbumKind.ROOT)
    tree.create_node(identifier=root.id, parent=None, tag=f'<root album>')

    _add_child_nodes(tree=tree, albums=albums, parent_id=root.id)

    return tree.show(stdout=False)


def _add_child_nodes(tree: Tree, albums: List[Album], parent_id: str) -> None:
    children = [album for album in albums if album.parent_album == parent_id]
    for child in children:
        tree.create_node(identifier=child.id, parent=parent_id, tag=_album_to_str(child))
        _add_child_nodes(tree, albums, child.id)


def _album_to_str(album: Album) -> str:
    if album.kind == AlbumKind.ROOT:
        description = '<root album>'
    elif album.kind == AlbumKind.USER_FOLDER:
        description = album.name
    else:
        timestamp = album.start_date.strftime("%Y-%m-%d %H:%M:%S") if album.start_date else '<no timestamp>'
        description = f'{timestamp}: {album.name}'

    return description
