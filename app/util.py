from datetime import datetime


def cocoa_timestamp_to_datetime(timestamp: str) -> datetime:
    """
    Converts a Cocoa timestamp to a human-readable datetime string.

    See https://stackoverflow.com/a/39542440 for more information.

    :param timestamp: Cocoa timestamp
    :return: Human-readable datetime string
    """
    unix_start = datetime(1970, 1, 1)
    cocoa_start = datetime(2001, 1, 1)

    delta = cocoa_start - unix_start

    return datetime.fromtimestamp(float(timestamp)) + delta