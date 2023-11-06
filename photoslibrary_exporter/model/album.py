from dataclasses import dataclass
from datetime import datetime
from enum import Enum


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
