import os


def db_path(library_path: str) -> str:
    """
    Gets the path of the Photos.sqlite database file.
    """
    return os.path.join(library_path, 'database', 'Photos.sqlite')