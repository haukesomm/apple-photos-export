# apple-photos-export

![Rust](https://img.shields.io/badge/Rust-d6a672?style=for-the-badge&logo=rust)

This utility program exports all images and videos from an Apple Photos Library to a local directory for backup
purposes. The library is accesses in a read-only and thus, does not modify it at all. 

> [!IMPORTANT]
> The project works by reverse-engineering the Apple Photos Library database and file structure. Thus, it is not
> guaranteed to work with future versions of the Photos app or at all. Use it at your own risk and always keep a backup
> of your library.

## Highlights

Features of this program include (but are not limited to):

- List all albums of the library
- Export all images and videos from the Photos Library to a local directory
- Different export album structures are supported, e.g.:
    - Flat (default)
    - By album
    - By year/month
    - By year/month/album
- Restore original filenames used when importing the assets into the library
- Sync mode: 
  - Skip assets that already have been exported
  - Delete existing assets that are no longer part of the library
- Dry-run mode to test the export without actually copying any files

## Compatibility

> [!NOTE]
> Currently, each version of this app only works with a specific combination of macOS and the Photos app.  
> Backwards compatibility to older versions is planned for future releases.

The following versions of the app are compatible with the following macOS/PhotosLibrary version:

| App version               | macOS Name | macOS Version   | Photos Version | Notes                                                                                                          |
|:--------------------------|:-----------|:----------------|:---------------|:---------------------------------------------------------------------------------------------------------------|
| `1.2.0-snapshot`          | Tahoe      | `26`            | `11.0`         |                                                                                                                |
| `1.1.0`                   | Tahoe      | `26`            | `11.0`         |                                                                                                                |
| `0.4.0`, `0.4.1`, `1.0.0` | Sequoia    | `15`            | `10.0`         |                                                                                                                |
| `0.3.0`                   | Sonoma     | `14.6`          | `9.0`          | The internal schema of the Photos app has changed, making this release incompatible with other Sonoma releases |
| `0.2.0`, `0.1.0`, `0.0.1` | Sonoma     | `14.0` - `14.5` | `9.0 `         |                                                                                                                |

## Changelog

For an overview of the changes made between each version, please have a look at the [CHANGELOG](CHANGELOG.md).

## Installation

Builds of this app are automatically published on release. You can find the binaries attached to the respective
GitHub release.

On macOS, the preferred way of installing this app is via Homebrew:

```shell
$ brew install haukesomm/repo/apple-photos-export
```

> [!IMPORTANT]
> Releases of this app are _not signed_.  
> macOS will complain about this and refuse to execute the app until you painstakingly white-list it via the system
> settings. Thanks for nothing, Apple.

> [!IMPORTANT]
> Currently, binaries of this app are only provided for __macOS on ARM__, i.e. M-series Macs.  
> Attempting to install the Homebrew formula on other platforms will result in an unspecific error.
> 
> Users on other OSs/architectures should be able to build the app locally on most other Unix-like operating systems,
> though.

Of course, the latest commits can alwas be built locally as well with no special magic required:

```shell
cargo build
```

### Listing albums

```shell
$ apple-photos-export list-albums <LIBRARY_PATH>
```

### Exporting assets

```shell
$ apple-photos-export <LIBRARY_PATH> export [OPTIONS] <OUTPUT_DIR>
```

Export configuration options:

```
-l, --group-by-album
        Group assets by album
-m, --group-by-year-month
        Group assets by year/month
-M, --group-by-year-month-album
        Group assets by year/month/album
-a, --include-by-album <INCLUDE_BY_ALBUM>...
        Include assets in the albums matching the given ids
-A, --exclude-by-album <EXCLUDE_BY_ALBUM>...
        Exclude assets in the albums matching the given ids
-v, --visible
        Only include assets that are not part of the 'hidden' album
-r, --restore-original-filenames
        Restore original filenames
-f, --flatten-albums
        Flatten album structure
-e, --include-edited
        Include edited versions of the assets if available
-E, --prefer-edited
        Prefer the edited version of the asset if available and fall back to the original otherwise
-d, --dry-run
        Dry run
-h, --help
        Print help
```

#### Examples

The following snippets show as an example how to export assets from a Photos Library and may be used as a starting
point.

> [!IMPORTANT]
> Remember to test the different configuration options using the `-d` flag (dry-run) before running any actual exports!

##### Full export structured by year/month/album

- Exports everything, including hidden assets
- Restores the original filenames used when importing the assets to the library
- Groups the assets in a `Year/Month/Album` structure
- Includes both the original and edited versions of each asset
- Flattens the album structure

```shell
$ apple-photos-export <LIBRARY_PATH> export <OUTPUT_DIR> -Mrfe
```

##### Only include assets that are part of one or more albums

- Exports all assets that _are_ part of the given albums (in this case `700` and `701`)
  - Album IDs can be obtained via the `list-albums` command

```shell
$ apple-photos-export <LIBRARY_PATH> export <OUTPUT_DIR> -a=700,701
```

##### Exclude assets that are part of at least one given album

- Exports all assets that _are not_ part of any of the given albums (in this case `700` and `701`)
  - Album IDs can be obtained via the `list-albums` command

```shell
$ apple-photos-export <LIBRARY_PATH> export <OUTPUT_DIR> -A=700,701
```