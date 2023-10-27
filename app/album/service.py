import os
from app.album import extractor as album_extractor, visualization as album_visualization


def print_album_tree(library_path: str) -> None:
    db_file_path = os.path.join(library_path, "database", "Photos.sqlite")

    albums = album_extractor.get_albums(db_file_path)
    tree = album_visualization.generate_ascii_album_tree(albums)
    print(tree)
