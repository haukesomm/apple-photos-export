import os.path
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import Optional


class AlbumKind(Enum):
    """
    Enum representing the individual album kinds that have been reverse-engineered from the
    library database.
    Each entry's value corresponds to the value used in the ``Photos.sqlite`` database.
    """
    ROOT = 3999
    USER_FOLDER = 4000
    USER_ALBUM = 2


@dataclass
class Album:
    """
    Data class representing an album.
    """
    id: str
    kind: AlbumKind
    parent_album: str
    name: str
    start_date: datetime


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
