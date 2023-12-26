import os
from dataclasses import dataclass

from apple_photos_export.export.strategy import ExportStrategy


@dataclass
class BaseContext:
    """
    Base context class that contains common app context properties used throughout the app.
    """
    library_path: str

    def photos_db_path(self) -> str:
        """
        Gets the path of the Photos.sqlite database file.
        """
        return os.path.join(self.library_path, 'database', 'Photos.sqlite')


@dataclass
class ExportContext(BaseContext):
    """
    Context class that contains base app context properties plus properties specific to the export action.
    """
    destination_path: str
    export_strategy: ExportStrategy
    restore_original_filenames: bool
    dry_run: bool
    flatten_albums: bool
    excluded_album_ids: list[str]
