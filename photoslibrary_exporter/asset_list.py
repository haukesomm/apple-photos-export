from typing import List

from photoslibrary_exporter import cocoa
from photoslibrary_exporter.context import ExportContext
from photoslibrary_exporter.model.asset import AssetWithAlbumInfo
from photoslibrary_exporter.repository import assets as asset_repo
from photoslibrary_exporter.repository.assets import AssetWithAlbumInfoDto


def get_assets_with_album_info(context: ExportContext) -> List[AssetWithAlbumInfo]:
    """
    Gets all assets from the library that should be exported and returns them as a list of ExportAsset objects.
    """

    def parse_dto(dto: AssetWithAlbumInfoDto) -> AssetWithAlbumInfo:
        if context.flatten_albums and dto.album_path:
            album_path = dto.album_path.removesuffix('/').split('/')[-1]
        else:
            album_path = dto.album_path

        if context.restore_original_filenames:
            asset_preferred_filename = dto.asset_original_filename
        else:
            asset_preferred_filename = dto.asset_filename

        asset_timestamp = cocoa.cocoa_timestamp_to_datetime(dto.asset_date) if dto.asset_date else None

        if dto.cocoa_album_start_date:
            album_timestamp = cocoa.cocoa_timestamp_to_datetime(dto.cocoa_album_start_date)
        else:
            album_timestamp = None

        return AssetWithAlbumInfo(
            asset_id=dto.asset_id,
            asset_directory=dto.asset_directory,
            asset_filename=dto.asset_filename,
            asset_original_filename=dto.asset_original_filename,
            asset_preferred_filename=asset_preferred_filename,
            asset_date=asset_timestamp,
            album_path=album_path,
            album_start_date=album_timestamp
        )

    export_asset_dtos = asset_repo.get_asset_data_with_album_info(context.photos_db_path(), context.excluded_album_ids)
    return list(map(parse_dto, export_asset_dtos))
