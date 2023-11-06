from dataclasses import dataclass

from photoslibrary_exporter.export.strategy import ExportStrategy


@dataclass
class Config:
    """
    Configuration object for the application.

    It contains various parameters that are used by the application.
    They are, as for now, the same as the command line arguments.
    """
    library_path: str
    destination_path: str
    export_strategy: ExportStrategy
    restore_original_filenames: bool
    dry_run: bool
    flatten_albums: bool
    excluded_album_ids: list[str]
