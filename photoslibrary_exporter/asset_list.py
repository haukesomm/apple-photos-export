from typing import List

from photoslibrary_exporter import cocoa, repo, library_file
from photoslibrary_exporter.config import Config
from photoslibrary_exporter.model import AssetWithAlbumInfo
from photoslibrary_exporter.repo import AssetWithAlbumInfoDto


def get_assets_with_album_info(config: Config) -> List[AssetWithAlbumInfo]:
    """
    Gets all assets from the library that should be exported and returns them as a list of ExportAsset objects.
    """

    db_file_path = library_file.get_photos_db_path(config.library_path)

    def parse_dto(dto: AssetWithAlbumInfoDto) -> AssetWithAlbumInfo:
        if config.flatten_albums and dto.album_path:
            album_path = dto.album_path.removesuffix('/').split('/')[-1]
        else:
            album_path = dto.album_path

        if config.restore_original_filenames:
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

    export_asset_dtos = repo.get_asset_data_with_album_info(db_file_path, config.excluded_album_ids)
    return list(map(parse_dto, export_asset_dtos))
