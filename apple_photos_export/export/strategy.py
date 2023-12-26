import os
from abc import ABC, abstractmethod
from datetime import datetime
from typing import Callable

from apple_photos_export.model.asset import AssetWithAlbumInfo


class ExportStrategy(ABC):
    """
    Abstract base class for export strategies.
    An export strategy is responsible for determining the relative output directory for a given asset.
    """

    @abstractmethod
    def get_relative_output_dir(self, asset: AssetWithAlbumInfo) -> str:
        """
        Returns the relative output directory for the given asset.
        Examples:
            - <output_dir>/2019/03/
            - <output_dir>/2019/03/MyAlbum/
            - <output_dir>/MyAlbum/
        """
        pass


class PlainExportStrategy(ExportStrategy):
    """
    Export strategy that exports all assets to the root of the export directory.
    """

    def get_relative_output_dir(self, asset: AssetWithAlbumInfo) -> str:
        return ''


class AlbumExportStrategy(ExportStrategy):
    """
    Export strategy that exports all assets grouped by their album hierarchy.
    """

    def __init__(self, flatten: bool):
        self._flatten = flatten

    def get_relative_output_dir(self, asset: AssetWithAlbumInfo) -> str:
        if self._flatten and asset.album_path:
            album_path = asset.album_path.removesuffix('/').split('/')[-1]
        else:
            album_path = asset.album_path or ''

        return album_path


class YearMonthExportStrategy(ExportStrategy):
    """
    Export strategy that exports all assets grouped by their year/month.

    By default, the asset date is used to determine the export path. Alternatively, a custom date selector can be
    provided, i.e. to use the album start date instead.
    """

    asset_date_selector: Callable[[AssetWithAlbumInfo], datetime.date] = lambda asset: asset.asset_date
    """
    Default date selector that returns the asset date.
    """

    album_date_selector: Callable[[AssetWithAlbumInfo], datetime.date] = \
        lambda asset: asset.album_start_date or asset.asset_date
    """
    Special date selector that returns the album start date if available, otherwise the asset date.
    """

    def __init__(self, date_selector: Callable[[AssetWithAlbumInfo], datetime.date] = asset_date_selector):
        self._date_selector = date_selector

    def get_relative_output_dir(self, asset: AssetWithAlbumInfo) -> str:
        return self._date_selector(asset).strftime('%Y/%m/')


class JoiningExportStrategy(ExportStrategy):
    """
    Export strategy that joins the paths of two or more other export strategies.
    """

    def __init__(self, *strategies: ExportStrategy):
        self._strategies = strategies

    def get_relative_output_dir(self, asset: AssetWithAlbumInfo) -> str:
        paths = map(lambda strategy: strategy.get_relative_output_dir(asset), self._strategies)
        return os.path.join('', *paths)
