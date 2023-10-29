import sqlite3
from dataclasses import dataclass
from typing import List, Any, Optional


@dataclass
class AlbumDto:
    id: str
    kind: int
    parent_album: str
    name: str
    cocoa_start_date: str


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
            FROM ZGENERICALBUM album
            WHERE album.ZKIND IN (2, 3999, 4000)
            ORDER BY album.ZSTARTDATE
            """
        )
        results = cursor.fetchall()

        return list(map(_album_dto_from_result, results))


def _album_dto_from_result(result: Any) -> AlbumDto:
    return AlbumDto(
        id=str(result[0]),
        kind=result[1],
        name=result[2],
        cocoa_start_date=result[3],
        parent_album=str(result[4])
    )


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
class ExportAssetDto:
    asset_id: str
    asset_directory: str
    asset_filename: str
    asset_original_filename: str
    album_path: str
    cocoa_album_start_date: Optional[str]


def get_export_asset_data(database_file_path: str) -> List[ExportAssetDto]:
    """
    Returns a list of all assets together with their original filenames and album information.

    :param database_file_path: Library database file path
    :return: List of export asset DTOs
    """
    with sqlite3.connect(f'file:{database_file_path}?mode=ro', uri=True) as conn:
        cursor = conn.cursor()
        cursor.execute(
            """
            WITH RECURSIVE ALBUM_PATH_CTE AS (
                SELECT Z_PK
                     , ZPARENTFOLDER
                     , '' AS path
                FROM ZGENERICALBUM
                WHERE ZGENERICALBUM.ZPARENTFOLDER IS NULL
            
            UNION ALL
            
                SELECT child.Z_PK
                     , child.ZPARENTFOLDER
                     , printf('%s%s%s', album.path, child.ZTITLE, '/') AS path
                FROM ZGENERICALBUM child
                JOIN ALBUM_PATH_CTE album
                  ON album.Z_PK = child.ZPARENTFOLDER
            )
            
            SELECT assets.Z_PK AS ASSET_ID
                 , assets.ZDIRECTORY AS ASSET_DIRECTORY
                 , assets.ZFILENAME AS ASSET_FILENAME
                 , attribs.ZORIGINALFILENAME AS ASSET_ORIGINAL_FILENAME
                 , album_path.path AS ALBUM_PATH
                 , album.ZSTARTDATE AS ALBUM_START_DATE
            FROM ZASSET assets
            LEFT JOIN ZADDITIONALASSETATTRIBUTES attribs ON assets.Z_PK = attribs.ZASSET
            LEFT JOIN Z_28ASSETS album_mapping ON assets.Z_PK = album_mapping.Z_3ASSETS
            LEFT JOIN ZGENERICALBUM album ON album_mapping.Z_28ALBUMS = album.Z_PK
            LEFT JOIN ALBUM_PATH_CTE album_path ON album.Z_PK = album_path.Z_PK
            """
        )
        results = cursor.fetchall()

        return list(map(_export_asset_dto_from_result, results))


def _export_asset_dto_from_result(result: Any) -> ExportAssetDto:
    return ExportAssetDto(
        asset_id=str(result[0]),
        asset_directory=result[1],
        asset_filename=result[2],
        asset_original_filename=result[3],
        album_path=result[4],
        cocoa_album_start_date=result[5]
    )
