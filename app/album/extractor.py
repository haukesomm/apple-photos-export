from datetime import datetime

from app.album import repo as album_repo
from app.album.repo import AlbumDto
from app.album.model import Album, AlbumKind


def get_albums(library_path: str) -> list[Album]:
    album_dtos = album_repo.get_albums(library_path)
    return list(map(_parse_album_dto, album_dtos))


def _parse_album_dto(dto: AlbumDto) -> Album:
    return Album(
        id=dto.id,
        kind=AlbumKind(dto.kind),
        parent_album=dto.parent_album,
        name=dto.name,
        start_date=_cocoa_timestamp_to_datetime(dto.cocoa_start_date) if dto.cocoa_start_date else None
    )


def _cocoa_timestamp_to_datetime(timestamp: str) -> datetime:
    """
    Converts a Cocoa timestamp to a human-readable datetime string.

    See https://stackoverflow.com/a/39542440 for more information.

    :param timestamp: Cocoa timestamp
    :return: Human-readable datetime string
    """
    unix_start = datetime(1970, 1, 1)
    cocoa_start = datetime(2001, 1, 1)

    delta = cocoa_start - unix_start

    return datetime.fromtimestamp(float(timestamp)) + delta
