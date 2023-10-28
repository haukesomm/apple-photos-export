import os
from app.album import extractor as album_extractor, visualization as album_visualization, repo as album_repo


def print_album_tree(library_path: str) -> None:
    db_file_path = os.path.join(library_path, "database", "Photos.sqlite")

    albums = album_extractor.get_albums(db_file_path)
    tree = album_visualization.generate_ascii_album_tree(albums)
    print(tree)

    asset_counts = album_repo.get_album_asset_counts(db_file_path)
    print(f'Total number of assets: {asset_counts.asset_count}')
    print(f'Number of assets not in an album: {asset_counts.asset_count_no_album}')
