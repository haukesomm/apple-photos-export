# Changelog

This file contains notable changes to this project in each release.
The most recent release is at the top.

## `0.2.0-snapshot`

### CLI

The following cli options/commands have been changed:

| Status  | Old             | New                     | Note                                                                 |
|---------|-----------------|-------------------------|----------------------------------------------------------------------|
| Renamed | `export-assets` | `export`                | Renamed to be shorter                                                |
| Renamed | `--include`     | `--include-albums`      | Renamed for better clarity                                           |
| Renamed | `--export`      | `--export-albums`       | Renamed for better clarity                                           |
| New     |                 | `-H`/`--include-hidden` | Includes hidden assets in the export. <br> _More information below._ |

### Asset export

- Add new feature to include hidden assets in the export. By default, hidden assets are not included.

### General

- Internal code cleanup and refactoring.

## `0.1.0`

Initial release.