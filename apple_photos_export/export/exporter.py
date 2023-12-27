import os
import shutil
from abc import ABC, abstractmethod
from typing import List

from colors import color

from apple_photos_export import library_file
from apple_photos_export.export.asset import AssetWithAlbumInfo
from apple_photos_export.export.properties import ExportProperties
from apple_photos_export.export.repo import get_asset_data_with_album_info
from apple_photos_export.export.strategy import ExportStrategy


class AssetExporter(ABC):
    """
    Abstract base class for asset exporters.

    An asset exporter is responsible for exporting a list of assets to a given destination path.
    """

    def __init__(self, strategy: ExportStrategy):
        self._strategy = strategy

    @abstractmethod
    def _export_single_asset(self, source_path: str, dest_path: str) -> None:
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

    def export(self, assets: List[AssetWithAlbumInfo], library_path: str, output_path: str,
               use_original_filename: bool) -> None:
        """
        Exports the given list of assets to the given destination path.
        """

        asset_count = len(assets)
        library_photos_path = os.path.join(library_path, 'originals')

        for index, asset in enumerate(assets):
            relative_dir = self._strategy.get_relative_output_dir(asset)
            filename = asset.asset_original_filename if use_original_filename else asset.asset_filename

            full_source_path = os.path.join(library_photos_path, asset.asset_path())
            full_dest_path = os.path.join(output_path, relative_dir, filename)

            print(
                ''.join([
                    color(f'({index + 1}/{asset_count})', fg='yellow'),
                    color(' Exporting ', fg='silver'),
                    color(asset.asset_filename, fg='grey'),
                    color(' to ', fg='silver'),
                    color(full_dest_path, fg='grey')
                ])
            )

            self._export_single_asset(full_source_path, full_dest_path)

        print()
        self._on_finished()


class DryRunAssetExporter(AssetExporter):
    """
    Asset exporter that does not actually export any files, but only prints the export operations to the console.
    It is used when the user specifies the --dry-run flag.
    """

    def _export_single_asset(self, source_path: str, dest_path: str) -> None:
        pass

    def _on_finished(self) -> None:
        print(color('Done. This was a dry run - no files were actually exported.', fg='fuchsia'))


class AssetExporterImpl(AssetExporter):
    """
    Asset exporter that actually exports the files.
    """

    def _export_single_asset(self, source_path: str, dest_path: str) -> None:
        # TODO: Handle errors (log) and continue
        os.makedirs(os.path.dirname(dest_path), exist_ok=True)
        shutil.copy(source_path, dest_path)

    def _on_finished(self) -> None:
        print(color('Done exporting assets.', fg='green'))


def export_assets(context: ExportProperties) -> None:
    """
    Exports all assets from the library to the given destination path.
    """

    database_path = library_file.db_path(context.library_path)
    assets = get_asset_data_with_album_info(database_path, context.excluded_album_ids)

    if context.dry_run:
        exporter = DryRunAssetExporter(context.export_strategy)
    else:
        exporter = AssetExporterImpl(context.export_strategy)

    exporter.export(assets, context.library_path, context.destination_path, context.restore_original_filenames)
