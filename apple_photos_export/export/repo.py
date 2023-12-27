import sqlite3
from dataclasses import dataclass
from typing import List, Any, Optional

from apple_photos_export import cocoa
from apple_photos_export.export.asset import AssetWithAlbumInfo


@dataclass
class AssetWithAlbumInfoDto:
    asset_id: str
    asset_directory: str
    asset_filename: str
    asset_original_filename: str
    asset_date: str
    album_path: Optional[str]
    cocoa_album_start_date: Optional[str]


def get_asset_data_with_album_info(database_file_path: str, excluded_ids: List[str]) -> List[AssetWithAlbumInfo]:
    """
    Returns a list of all assets together with their original filenames and album information.

    :param database_file_path: Library database file path
    :param excluded_ids: List of album ids that should be excluded from the export
    :return: List of assets with their album information
    """

    with sqlite3.connect(f'file:{database_file_path}?mode=ro', uri=True) as conn:
        conn.row_factory = sqlite3.Row

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
                WHERE child.ZTRASHEDSTATE = 0
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
            WHERE (album.ZKIND IS NULL OR album.ZKIND IN (2, 3999, 4000))
               AND (album.Z_PK IS NULL OR album.Z_PK NOT IN ({', '.join('?' for _ in excluded_ids)}))
            """

        cursor = conn.cursor()
        cursor.execute(sql, tuple(excluded_ids))
        results = cursor.fetchall()

        def parse_result(result: Any) -> AssetWithAlbumInfo:
            return AssetWithAlbumInfo(
                asset_id=result['ASSET_ID'],
                asset_directory=result['ASSET_DIRECTORY'],
                asset_filename=result['ASSET_FILENAME'],
                asset_original_filename=result['ASSET_ORIGINAL_FILENAME'],
                asset_date=cocoa.timestamp_to_datetime(result['ASSET_DATE']),
                album_path=result['ALBUM_PATH'],
                album_start_date=cocoa.timestamp_to_datetime(time) if (time := result['ALBUM_START_DATE']) else None
            )

        return list(map(parse_result, results))
