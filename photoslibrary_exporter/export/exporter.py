import os
import shutil
from abc import ABC, abstractmethod
from typing import List

from colors import color

from photoslibrary_exporter import asset_list, library_file
from photoslibrary_exporter.export.strategy import ExportStrategy
from photoslibrary_exporter.model import ExportAsset, AssetWithAlbumInfo


class AssetExporter(ABC):
    """
    Abstract base class for asset exporters.

    An asset exporter is responsible for exporting a list of assets to a given destination path.
    """

    def __init__(self, strategy: ExportStrategy):
        self._strategy = strategy

    @abstractmethod
    def _export_single_asset(self, export_asset: ExportAsset) -> None:
        """
        Abstract method that must be implemented by subclasses and that performs the actual export.
        This method is called by the public export() method.
        """
        pass

    @abstractmethod
    def _on_finished(self) -> None:
        """
        Abstract method that is called after the export has finished.
        """
        pass

    def export(self, assets: List[AssetWithAlbumInfo], library_path: str, output_path: str) -> None:
        """
        Exports the given list of assets to the given destination path.
        """
        asset_count = len(assets)
        library_photos_path = os.path.join(library_path, 'originals')

        for index, asset in enumerate(assets):
            export_asset = self._strategy.get_export_asset(asset, library_photos_path, output_path)

            print(
                ''.join([
                    color(f'({index + 1}/{asset_count})', fg='yellow'),
                    color(' Exporting ', fg='silver'),
                    color(asset.asset_filename, fg='grey'),
                    color(' to ', fg='silver'),
                    color(export_asset.exported_asset_path, fg='grey')
                ])
            )

            self._export_single_asset(export_asset)

        print()
        self._on_finished()


class DryRunAssetExporter(AssetExporter):
    """
    Asset exporter that does not actually export any files, but only prints the export operations to the console.
    It is used when the user specifies the --dry-run flag.
    """

    def _export_single_asset(self, export_asset: ExportAsset) -> None:
        pass

    def _on_finished(self) -> None:
        print(color('Done. This was a dry run - no files were actually exported.', fg='fuchsia'))


class AssetExporterImpl(AssetExporter):
    """
    Asset exporter that actually exports the files.
    """

    def _export_single_asset(self, export_asset: ExportAsset) -> None:
        # TODO: Handle errors (log) and continue
        os.makedirs(os.path.dirname(export_asset.exported_asset_path), exist_ok=True)
        shutil.copy(export_asset.library_asset_path, export_asset.exported_asset_path)

    def _on_finished(self) -> None:
        print(color('Done exporting assets.', fg='green'))


def export_assets(library_file_path: str, strategy: ExportStrategy, restore_original_filename: bool, dry_run: bool, flatten_albums: bool, excluded_ids: List[str], output_path: str) -> None:
    """
    Exports all assets from the library to the given destination path.
    """

    db_file_path = library_file.get_photos_db_path(library_file_path)
    assets = asset_list.get_assets_with_album_info(
        db_file_path, restore_original_filename, flatten_albums, excluded_ids)

    if dry_run:
        exporter = DryRunAssetExporter(strategy)
    else:
        exporter = AssetExporterImpl(strategy)

    exporter.export(assets, library_file_path, output_path)
