from typing import List

from photoslibrary_exporter import cocoa, repo
from photoslibrary_exporter.model import AssetWithAlbumInfo
from photoslibrary_exporter.repo import AssetWithAlbumInfoDto


def get_assets_with_album_info(db_file_path: str, restore_original_filename: bool, flatten_albums: bool, excluded_ids: List[str]) -> List[AssetWithAlbumInfo]:
    """
    Gets all assets from the library that should be exported and returns them as a list of ExportAsset objects.
    """

    def parse_dto(dto: AssetWithAlbumInfoDto) -> AssetWithAlbumInfo:
        if flatten_albums:
            album_path = dto.album_path.removesuffix('/').split('/')[-1]
        else:
            album_path = dto.album_path

        return AssetWithAlbumInfo(
            asset_id=dto.asset_id,
            asset_directory=dto.asset_directory,
            asset_filename=dto.asset_filename,
            asset_original_filename=dto.asset_original_filename,
            asset_preferred_filename=dto.asset_original_filename if restore_original_filename else dto.asset_filename,
            asset_date=cocoa.cocoa_timestamp_to_datetime(dto.asset_date) if dto.asset_date else None,
            album_path=album_path,
            album_start_date=cocoa.cocoa_timestamp_to_datetime(
                dto.cocoa_album_start_date) if dto.cocoa_album_start_date else None
        )

    export_asset_dtos = repo.get_asset_data_with_album_info(db_file_path, excluded_ids)
    return list(map(parse_dto, export_asset_dtos))
