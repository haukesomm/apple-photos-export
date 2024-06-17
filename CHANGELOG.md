# Changelog

The following is a list of changes made to the project in reverse chronological order.

## `0.2.0-snapshot`

- Add support for offline-only libraries. Previously, the app only supported icloud-enabled libraries.
- Rename `-e`/`--exclude-albums` flag to `-x`/`--exclude-albums`.
- Add new export flags:
    - `-e`/`--include-edited` to include edited asset versions in the export if available.
    - `-E`/`--only-edited` to always export the edited version of an asset if available.
- Internal refactoring of the export command's logic

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