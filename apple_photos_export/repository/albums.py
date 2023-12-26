import sqlite3
from dataclasses import dataclass
from typing import List, Any

from apple_photos_export import cocoa
from apple_photos_export.model.album import Album, AlbumKind


def get_albums(database_file_path: str) -> List[Album]:
    """
    Returns a list of all user-created albums in the database.
    System albums are not included.

    :param database_file_path: Path of the Photos.sqlite file
    :return: List of all user-created albums
    """
    with sqlite3.connect(f'file:{database_file_path}?mode=ro', uri=True) as conn:
        conn.row_factory = sqlite3.Row

        cursor = conn.cursor()
        cursor.execute(
            """
            SELECT album.Z_PK
                 , album.ZKIND
                 , album.ZTITLE
                 , album.ZSTARTDATE
                 , album.ZPARENTFOLDER
                 , (
                        SELECT COUNT(*)
                        FROM Z_28ASSETS mapping
                        WHERE mapping.Z_28ALBUMS = album.Z_PK
                   ) AS ASSET_COUNT
            FROM ZGENERICALBUM album
            WHERE album.ZKIND IN (2, 3999, 4000) AND album.ZTRASHEDSTATE = 0
            ORDER BY album.ZSTARTDATE;
            """
        )
        results = cursor.fetchall()

        def parse_result(result: Any) -> Album:
            return Album(
                id=result['Z_PK'],
                kind=AlbumKind(result['ZKIND']),
                parent_album=result['ZPARENTFOLDER'],
                name=result['ZTITLE'],
                start_date=cocoa.timestamp_to_datetime(start) if (start := result['ZSTARTDATE']) else None,
                asset_count=result['ASSET_COUNT']
            )

        return list(map(parse_result, results))
