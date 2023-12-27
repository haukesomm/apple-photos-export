import sqlite3
from typing import List, Any

from apple_photos_export import cocoa
from apple_photos_export.albums.album import Album, AlbumKind, AssetCount


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


def get_album_asset_counts(database_file_path: str) -> AssetCount:
    """
    Returns the number of assets in the database and the number of assets that are not part of any album.

    :param database_file_path: Library database file path
    :return: Asset count DTO
    """
    with sqlite3.connect(f'file:{database_file_path}?mode=ro', uri=True) as conn:
        conn.row_factory = sqlite3.Row

        cursor = conn.cursor()
        cursor.execute(
            """
            select COUNT(assets.Z_PK) as ASSET_CNT_TOTAL
                 , COUNT(album_mapping.Z_3ASSETS) as ASSET_CNT_ALBUM
            from ZASSET assets
            left join Z_28ASSETS album_mapping on assets.Z_PK = album_mapping.Z_3ASSETS
            """
        )
        result = cursor.fetchall()[0]

        return AssetCount(
            total=result['ASSET_CNT_TOTAL'],
            album=result['ASSET_CNT_ALBUM']
        )
