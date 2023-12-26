import os
from dataclasses import dataclass

from apple_photos_export.export.strategy import ExportStrategy


@dataclass
class ExportProperties:
    """
    Context class that contains base app context properties plus properties specific to the export action.
    """
    library_path: str
    destination_path: str
    export_strategy: ExportStrategy
    restore_original_filenames: bool
    dry_run: bool
    flatten_albums: bool
    excluded_album_ids: list[str]
