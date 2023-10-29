import os
import shutil
from abc import ABC, abstractmethod
from typing import List

from colors import color

from photoslibrary_exporter import asset_list, library_file
from photoslibrary_exporter.model import ExportAsset


class AssetExporter(ABC):
    """
    Abstract base class for asset exporters.

    An asset exporter is responsible for exporting a list of assets to a given destination path.
    """

    @abstractmethod
    def _export_single_asset(self, index: int, last: int, asset_path: str, output_path: str) -> None:
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

    def export(self, destination_path: str, library_file_path: str, export_asset_list: List[ExportAsset]) -> None:
        """
        Exports the given list of assets to the given destination path.
        """
        asset_count = len(export_asset_list)

        for index, asset in enumerate(export_asset_list):
            if index == 100:
                break

            asset_path = os.path.join(library_file_path, 'originals', asset.asset_directory, asset.asset_filename)
            output_path = os.path.join(
                destination_path,
                asset.album_path if asset.album_path else '',
                asset.dest_filename
            )

            print(
                ''.join([
                    color(f'({index + 1}/{asset_count})', fg='yellow'),
                    color(' Exporting ', fg='silver'),
                    color(asset.asset_filename, fg='grey'),
                    color(' to ', fg='silver'),
                    color(output_path, fg='grey')
                ])
            )

            self._export_single_asset(index, asset_count, asset_path, output_path)

        print()
        self._on_finished()


class DryRunAssetExporter(AssetExporter):
    """
    Asset exporter that does not actually export any files, but only prints the export operations to the console.
    It is used when the user specifies the --dry-run flag.
    """

    def _export_single_asset(self, index: int, last: int, asset_path: str, output_path: str) -> None:
        pass

    def _on_finished(self) -> None:
        print(color('Done. This was a dry run - no files were actually exported.', fg='fuchsia'))


class AssetExporterImpl(AssetExporter):
    """
    Asset exporter that actually exports the files.
    """

    def _export_single_asset(self, index: int, last: int, asset_path: str, output_path: str) -> None:
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        shutil.copy(asset_path, output_path)

    def _on_finished(self) -> None:
        print(color('Done exporting assets.', fg='green'))


def export_assets(library_file_path: str, destination_path: str, restore_filenames: bool, dry_run: bool) -> None:
    """
    Exports all assets from the library to the given destination path.
    """

    db_file_path = library_file.get_photos_db_path(library_file_path)
    assets = asset_list.get_export_assets(db_file_path, restore_filenames)

    if dry_run:
        exporter = DryRunAssetExporter()
    else:
        exporter = AssetExporterImpl()

    exporter.export(destination_path, library_file_path, assets)
