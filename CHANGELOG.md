# Changelog

The following is a list of changes made to the project in reverse chronological order.

## `0.2.0-snapshot`

### Overview

- Add `changelog` command to display this changelog from the CLI.
- Add new feature to include hidden assets in the export. By default, hidden assets are not included.
- Rearrange arguments: The library path is no longer the first argument. 
Instead, it is now the first argument after the subcommand.
- Internal code cleanup and refactoring.

### CLI

The following table lists the changes made to the CLI.

| Status  | Type       | Old             | New                     | Note                                                                 |
|---------|------------|-----------------|-------------------------|----------------------------------------------------------------------|
| Added   | Subcommand |                 | `changelog`             | Displays this changelog.                                             |
| Added   | Flag       |                 | `-H`/`--include-hidden` | Includes hidden assets in the export. <br> _More information below._ |
| Renamed | Subcommand | `export-assets` | `export`                | Renamed to be shorter                                                |
| Renamed | Flag       | `--include`     | `--include-albums`      | Renamed for better clarity                                           |
| Renamed | Flag       | `--export`      | `--export-albums`       | Renamed for better clarity                                           |

## `0.1.0`

Initial release.