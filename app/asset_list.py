from typing import List

from app import util, repo
from app.model import ExportAsset
from app.repo import ExportAssetDto


def get_export_assets(db_file_path: str, restore_filenames: bool) -> List[ExportAsset]:
    """
    Gets all assets from the library that should be exported and returns them as a list of ExportAsset objects.
    """

    export_asset_dtos = repo.get_export_asset_data(db_file_path)
    return [_export_asset_from_dto(dto, restore_filenames) for dto in export_asset_dtos]


def _export_asset_from_dto(dto: ExportAssetDto, restore_filenames: bool) -> ExportAsset:
    filename = dto.asset_original_filename if restore_filenames else dto.asset_filename
    timestamp = util.cocoa_timestamp_to_datetime(dto.cocoa_album_start_date) if dto.cocoa_album_start_date else None

    return ExportAsset(
        asset_id=dto.asset_id,
        asset_directory=dto.asset_directory,
        asset_filename=dto.asset_filename,
        dest_filename=filename,
        album_path=dto.album_path,
        album_timestamp_start=timestamp
    )
