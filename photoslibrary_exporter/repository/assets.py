import sqlite3
from dataclasses import dataclass
from typing import List, Any, Optional

from photoslibrary_exporter.model.album import AlbumKind


@dataclass
class AssetCountDto:
    asset_count: int
    asset_count_no_album: int


def get_album_asset_counts(database_file_path: str) -> AssetCountDto:
    """
    Returns the number of assets in the database and the number of assets that are not part of any album.

    :param database_file_path: Library database file path
    :return: Asset count DTO
    """
    with sqlite3.connect(f'file:{database_file_path}?mode=ro', uri=True) as conn:
        cursor = conn.cursor()
        cursor.execute(
            """
            SELECT COUNT(assets.Z_PK) AS ASSET_CNT
                 , COUNT(assets.Z_PK) - COUNT(album_mapping.Z_3ASSETS) AS ASSET_CNT_NO_ALBUM
            FROM ZASSET assets
            LEFT JOIN Z_28ASSETS album_mapping ON assets.Z_PK = album_mapping.Z_3ASSETS
            """
        )
        result = cursor.fetchall()[0]

        return AssetCountDto(
            asset_count=result[0],
            asset_count_no_album=result[1]
        )


@dataclass
class AssetWithAlbumInfoDto:
    asset_id: str
    asset_directory: str
    asset_filename: str
    asset_original_filename: str
    asset_date: str
    album_path: Optional[str]
    cocoa_album_start_date: Optional[str]


def get_asset_data_with_album_info(database_file_path: str, excluded_ids: List[str]) -> List[AssetWithAlbumInfoDto]:
    """
    Returns a list of all assets together with their original filenames and album information.

    :param database_file_path: Library database file path
    :param excluded_ids: List of album ids that should be excluded from the export
    :return: List of export asset DTOs
    """

    with sqlite3.connect(f'file:{database_file_path}?mode=ro', uri=True) as conn:
        cursor = conn.cursor()

        allowed_album_kinds = [k.value for k in (AlbumKind.ROOT, AlbumKind.USER_ALBUM, AlbumKind.USER_FOLDER)]

        sql = f"""
            WITH RECURSIVE ALBUM_PATH_CTE AS (
                SELECT Z_PK
                     , ZPARENTFOLDER
                     , '' AS path
                FROM ZGENERICALBUM
                WHERE ZGENERICALBUM.ZPARENTFOLDER IS NULL
            
            UNION ALL
            
                SELECT child.Z_PK
                     , child.ZPARENTFOLDER
                     , printf('%s%s/', album.path, child.ZTITLE) AS path
                FROM ZGENERICALBUM child
                JOIN ALBUM_PATH_CTE album
                  ON album.Z_PK = child.ZPARENTFOLDER
            )
            
            SELECT assets.Z_PK AS ASSET_ID
                 , assets.ZDIRECTORY AS ASSET_DIRECTORY
                 , assets.ZFILENAME AS ASSET_FILENAME
                 , attribs.ZORIGINALFILENAME AS ASSET_ORIGINAL_FILENAME
                 , assets.ZDATECREATED AS ASSET_DATE
                 , album_path.path AS ALBUM_PATH
                 , album.ZSTARTDATE AS ALBUM_START_DATE
            FROM ZASSET assets
            LEFT JOIN ZADDITIONALASSETATTRIBUTES attribs ON assets.Z_PK = attribs.ZASSET
            LEFT JOIN Z_28ASSETS album_mapping ON assets.Z_PK = album_mapping.Z_3ASSETS
            LEFT JOIN ZGENERICALBUM album ON album_mapping.Z_28ALBUMS = album.Z_PK
            LEFT JOIN ALBUM_PATH_CTE album_path ON album.Z_PK = album_path.Z_PK
            WHERE (album.ZKIND IS NULL OR album.ZKIND IN ({', '.join('?' for _ in allowed_album_kinds)}))
               AND (album.Z_PK IS NULL OR album.Z_PK NOT IN ({', '.join('?' for _ in excluded_ids)}))
            """

        cursor.execute(sql, tuple(allowed_album_kinds + excluded_ids))

        results = cursor.fetchall()

        def parse_result(result: Any) -> AssetWithAlbumInfoDto:
            return AssetWithAlbumInfoDto(
                asset_id=str(result[0]),
                asset_directory=result[1],
                asset_filename=result[2],
                asset_original_filename=result[3],
                asset_date=result[4],
                album_path=result[5],
                cocoa_album_start_date=result[6]
            )

        return list(map(parse_result, results))
