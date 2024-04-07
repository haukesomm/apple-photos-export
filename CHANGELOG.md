# Changelog

The following is a list of changes made to the project in reverse chronological order.

## `0.2.0-snapshot`

### Overview

- Use [diesel](https://diesel.rs) ORM for database access.
- Add `changelog` subcommand to display this changelog from the CLI.
- Rename `export-assets` subcommand to `export`.
- Rename `--include` flag to `--include-albums`.
- Rename `--exclude` flag to `--exclude-albums`.
- Add `-H`/`--include-hidden` export flag to include hidden assets in the export. By default, hidden assets are not included.
- Loads of refactoring and code cleanup.

## `0.1.0`

Initial release.