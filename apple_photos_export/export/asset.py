import os
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
    asset_date: datetime
    album_path: str
    album_start_date: Optional[datetime]

    def asset_path(self):
        return os.path.join(self.asset_directory, self.asset_filename)