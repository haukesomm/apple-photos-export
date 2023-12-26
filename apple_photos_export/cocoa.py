from datetime import datetime


def timestamp_to_datetime(timestamp: str) -> datetime:
    """
    Converts a Cocoa timestamp to a standard datetime object.

    Cocoa is a libray by Apple that is used to develop macOS and iOS applications.
    It's timestamp format stores the number of seconds since 2001-01-01 00:00:00 UTC.

    See https://stackoverflow.com/a/39542440 for more information.

    :param timestamp: Cocoa timestamp
    :return: Human-readable datetime string
    """
    unix_start = datetime(1970, 1, 1)
    cocoa_start = datetime(2001, 1, 1)

    delta = cocoa_start - unix_start

    return datetime.fromtimestamp(float(timestamp)) + delta
