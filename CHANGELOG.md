# Changelog

The following is a list of changes made to the project in reverse chronological order.

## `0.5.0`

(no changes yet)

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
