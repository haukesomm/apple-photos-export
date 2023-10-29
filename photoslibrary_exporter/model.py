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
class ExportAsset:
    """
    Data class representing an asset that is to be exported.
    """
    asset_id: str
    asset_directory: str
    asset_filename: str
    dest_filename: str
    album_path: Optional[str]
    album_timestamp_start: Optional[datetime]
