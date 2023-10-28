import os
from app import visualization as album_visualization, album_list, asset_list, repo as album_repo, export


class ExporterService:
    """
    Service class offering various methods to export photos from the Photos library.
    """

    def __init__(self, library_file_path: str):
        self.library_file_path = library_file_path
        self.db_file_path = os.path.join(library_file_path, "database", "Photos.sqlite")

    def print_album_tree(self) -> None:
        """
        Gets all albums from the library and prints them as an ASCII tree.
        """

        albums = album_list.get_albums(self.db_file_path)
        tree = album_visualization.generate_ascii_album_tree(albums)
        print(tree)

        asset_counts = album_repo.get_album_asset_counts(self.db_file_path)
        print(f'Total number of assets: {asset_counts.asset_count}')
        print(f'Number of assets not in an album: {asset_counts.asset_count_no_album}')

    def export_assets(self, destination_path: str, restore_filenames: bool, dry_run: bool) -> None:
        """
        Exports all assets from the library to the given destination path.
        """

        export_assets = asset_list.get_export_assets(self.db_file_path, restore_filenames)

        if dry_run:
            exporter = export.DryRunAssetExporter()
        else:
            exporter = export.AssetExporterImpl()

        exporter.export(destination_path, self.library_file_path, export_assets)
