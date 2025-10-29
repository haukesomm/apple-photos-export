# Changelog

The following is a list of changes made to the project in reverse chronological order.

## `1.1.0-SNAPSHOT`

### New output options

| Flag                   | Description                                                                        |
|------------------------|------------------------------------------------------------------------------------|
| `-s`/`--skip-existing` | Don't copy files that already exist in the output directory.                       |
| `--delete`             | Delete files from the output directory that are not present in the Photos library. |

## `1.0.0`

- Complete rewrite from the ground up
- The library path is now supplied _before_ the subcommand in the CLI: 
`apple-photos-export <LIBRARY_PATH> <SUBCOMMAND> [FLAGS]`
- The `changelog` subcommand has been removed as the changelog can easily be viewed on GitHub

Updated arguments/flags:

| old                                       | new                           | comment                                                         |
|-------------------------------------------|-------------------------------|-----------------------------------------------------------------|
| `-H/--include-hidden`, `--must-be-hidden` | `-v/--visible`                | have been combined                                              |
| `-a/--by-album`                           | `-l/--by-album`               | has been renamed                                                |
| `-i/--include-albums`                     | `-a/--include-by-album`       | has been renamed                                                |
| `-x/--exclude-albums`                     | `-A/--exclude-by-album`       | has been renamed                                                |
| `--by-album`                              | `--group-by-album`            | has been renamed                                                |
| `--by-year-month`                         | `--group-by-year-month`       | has been renamed                                                |
| `--by-year-month-album`                   | `--group-by-year-month-album` | has been renamed                                                |
| `-E/--only-edited`                        | `-E/--prefer-edited`          | __behavior has changed__ <br> see `--help` for more information |

### Technical details

- `diesel` has once again been replaced with `rusqlite` for database access
  - This is due to the simplicity of the database schema and the fact that `diesel` was overkill for this project

## `0.4.1`

- Adds support for `.bmp` and `.raf` files (see [#1](https://github.com/haukesomm/apple-photos-export/issues/1))
- Adds support for JPEG files stored as `.jpg` internally that could not be exported due to the exporter failing to
  determine their UTI (see [#1](https://github.com/haukesomm/apple-photos-export/issues/1))

## `0.4.0`

- Target macOS 15 with Photos 10.0

## `0.3.0`

- Target macOS 14.6 with Photos 9.0
  - Even though the version remained the same, the internal schema of the Photos database changed with the above 
    version
- Improve error handling and error messages
  - Errors are now exported to an error log file in the export directory
  - Internal error handling has been improved
- Rename database models to be better distinguishable from the library models

## `0.2.0`

- Add support for offline-only libraries. Previously, the app only supported icloud-enabled libraries.
- Rename `-e`/`--exclude-albums` flag to `-x`/`--exclude-albums`.
- Add new export flags:
    - `-e`/`--include-edited` to include edited asset versions in the export if available.
    - `-E`/`--only-edited` to always export the edited version of an asset if available.
- Internal refactoring of the export command's logic
- Even more internal cleanup and refactoring

## `0.1.0`

- Use [diesel](https://diesel.rs) ORM for database access.
- Proper handling of assets that have been offloaded to iCloud
- Add `changelog` subcommand to display this changelog from the CLI.
- Rename `export-assets` subcommand to `export`.
- Rename `--include` flag to `--include-albums`.
- Rename `--exclude` flag to `--exclude-albums`.
- Add new flags to controll whether hidden assets are included or not:
  - Add `-H`/`--include-hidden` export flag to include hidden assets in the export.
  - Add `--must-be-hidden` flag to exclusively export hidden assets.
  - By default, hidden assets are not included.
- Loads of refactoring and code cleanup.

## `0.0.1`

Initial release.
