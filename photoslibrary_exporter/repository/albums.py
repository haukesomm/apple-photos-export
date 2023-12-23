import sqlite3
from dataclasses import dataclass
from typing import List, Any


@dataclass
class AlbumDto:
    id: str
    kind: int
    parent_album: str
    name: str
    cocoa_start_date: str
    asset_count: int


def get_albums(database_file_path: str) -> List[AlbumDto]:
    """
    Returns a list of all user-created albums in the database.
    System albums are not included.

    :param database_file_path: Path of the Photos.sqlite file
    :return: List of all user-created albums
    """
    with sqlite3.connect(f'file:{database_file_path}?mode=ro', uri=True) as conn:
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

        def parse_result(result: Any) -> AlbumDto:
            return AlbumDto(
                id=str(result[0]),
                kind=result[1],
                name=result[2],
                cocoa_start_date=result[3],
                parent_album=str(result[4]),
                asset_count=result[5]
            )

        return list(map(parse_result, results))
