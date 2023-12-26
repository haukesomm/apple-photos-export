import os
from abc import ABC, abstractmethod

from apple_photos_export.model.asset import ExportAsset, AssetWithAlbumInfo


class ExportStrategy(ABC):
    """
    Abstract base class for export strategies.

    An export strategy is responsible for determining the export path of a given asset.
    """

    @abstractmethod
    def get_export_asset(self, asset: AssetWithAlbumInfo, library_photos_path: str, output_path: str) -> ExportAsset:
        pass


class PlainExportStrategy(ExportStrategy):
    """
    Export strategy that exports all assets to the root of the export directory.
    """

    def get_export_asset(self, asset: AssetWithAlbumInfo, library_photos_path: str, output_path: str) -> ExportAsset:
        return ExportAsset(
            asset_id=asset.asset_id,
            library_asset_path=os.path.join(library_photos_path, asset.asset_path()),
            exported_asset_path=os.path.join(output_path, asset.asset_preferred_filename)
        )


class AlbumExportStrategy(ExportStrategy):
    """
    Export strategy that exports all assets grouped by their album hierarchy.
    """

    def get_export_asset(self, asset: AssetWithAlbumInfo, library_photos_path: str, output_path: str) -> ExportAsset:
        return ExportAsset(
            asset_id=asset.asset_id,
            library_asset_path=os.path.join(library_photos_path, asset.asset_path()),
            exported_asset_path=os.path.join(output_path, asset.album_path or '', asset.asset_preferred_filename)
        )


class YearMonthExportStrategy(ExportStrategy):
    """
    Export strategy that exports all assets grouped by their year/month.
    """

    def get_export_asset(self, asset: AssetWithAlbumInfo, library_photos_path: str, output_path: str) -> ExportAsset:
        return ExportAsset(
            asset_id=asset.asset_id,
            library_asset_path=os.path.join(library_photos_path, asset.asset_path()),
            exported_asset_path=os.path.join(output_path,
                                             asset.asset_date.strftime('%Y/%m/'),
                                             asset.asset_preferred_filename)
        )


class YearMonthAlbumExportStrategy(ExportStrategy):
    """
    Export strategy that exports all assets grouped by their year/month and album hierarchy.
    """

    def get_export_asset(self, asset: AssetWithAlbumInfo, library_photos_path: str, output_path: str) -> ExportAsset:
        # If the asset is not in an album, use the asset date to determine the export path.
        # This means that if an asset is in an album, it will be in the year/month folder of the album's start date.
        sorting_date = asset.album_start_date or asset.asset_date

        return ExportAsset(
            asset_id=asset.asset_id,
            library_asset_path=os.path.join(library_photos_path, asset.asset_path()),
            exported_asset_path=os.path.join(output_path,
                                             sorting_date.strftime('%Y/%m/'),
                                             asset.album_path or '',
                                             asset.asset_preferred_filename)
        )
