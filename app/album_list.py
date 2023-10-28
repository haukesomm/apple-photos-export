from app import util, repo as album_repo
from app.repo import AlbumDto
from app.model import Album, AlbumKind


def get_albums(library_path: str) -> list[Album]:
    """
    Gets all albums from the library and returns them as a list of Album objects.
    """

    album_dtos = album_repo.get_albums(library_path)
    return list(map(_parse_album_dto, album_dtos))


def _parse_album_dto(dto: AlbumDto) -> Album:
    return Album(
        id=dto.id,
        kind=AlbumKind(dto.kind),
        parent_album=dto.parent_album,
        name=dto.name,
        start_date=util.cocoa_timestamp_to_datetime(dto.cocoa_start_date) if dto.cocoa_start_date else None
    )
