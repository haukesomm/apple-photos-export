import os.path
from dataclasses import dataclass
from datetime import datetime
from typing import Optional


@dataclass
class AssetWithAlbumInfo:
    """
    Data class representing an asset from the library, including information about the album it is in.
    """
    asset_id: str
    asset_directory: str
    asset_filename: str
    asset_original_filename: str
    asset_preferred_filename: str
    asset_date: datetime
    album_path: str
    album_start_date: Optional[datetime]

    def asset_path(self):
        return os.path.join(self.asset_directory, self.asset_filename)


@dataclass
class ExportAsset:
    """
    Data class representing an asset that is to be exported.

    The paths have previously been computed based on the export strategy the user has chosen.
    """
    asset_id: str
    library_asset_path: str
    exported_asset_path: str
