# apple-photos-export

![Rust](https://img.shields.io/badge/Rust-d6a672?style=for-the-badge&logo=rust)

This program exports all images and videos from an Apple Photos Library to a local directory, making the files easily
accessible without the need to use the native Photos app.
It is intended for backup purposes and does not modify the library in any way.

> [!IMPORTANT]
> The project works by reverse-engineering the Apple Photos Library database and file structure. Thus, it is not
> guaranteed to work with future versions of the Photos app or at all. Use it at your own risk and always keep a backup
> of your library.

Quick links to installation and usage instructions:

- [Building and Running locally](#building-and-running-locally)
- [Installation via Homebrew](#installation-via-homebrew)

## Highlights

- Lists all albums of the library as an ascii tree
- Exports all images and videos from the Photos Library to a local directory
- Different export album structures are supported (overview below)
    - Flat (default)
    - Album
    - Year/Month
    - Year/Month/Album
- Optionally restores the original file names that were used when importing the files into the library
- Dry-run mode to test the export without actually copying any files

## Usage

### Building and Running locally

```shell
$ cargo build --release
$ ./target/release/apple-photos-export --help
```

### Installation via Homebrew

```shell
$ brew install haukesomm/repo/apple-photos-export
```

OR

```shell
$ brew tap haukesomm/repo
$ brew install apple-photos-export
```

### Listing albums

```shell
$ apple-photos-export list-albums <LIBRARY_PATH>
```

### Exporting assets

```shell
$ apple-photos-export export [OPTIONS] <LIBRARY_PATH> <OUTPUT_DIR>
```

<details>
    <summary>Configuration options</summary>

```
-a, --by-album                       Group assets by album
-m, --by-year-month                  Group assets by year/month
-M, --by-year-month-album            Group assets by year/month/album
-i, --include-albums [<INCLUDE>...]  Include assets in the albums matching the given ids
-x, --exclude-albums <EXCLUDE>...    Exclude assets in the albums matching the given ids
-H, --include-hidden                 Include hidden assets
--must-be-hidden                 Assets must be hidden
-r, --restore-original-filenames     Restore original filenames
-f, --flatten-albums                 Flatten album structure
-e, --include-edited                 Include edited versions of the assets if available
-E, --only-edited                    Always export the edited version of an asset if available
-d, --dry-run                        Dry run
-h, --help                           Print help
```

</details>

#### Examples

The following snippets show as an example how to export assets from a Photos Library and may be used as a starting
point.

> [!IMPORTANT]
> Remember to test the different configuration options using the `-d` flag (dry-run) before running any actual exports!


<details>
    <summary>Snippets</summary>

##### Full export

- Exports everything, including hidden assets
    - Restores the original filenames used when importing the assets to the library
    - Groups the assets in a `Year/Month/Album` structure
    - Includes both the original and edited versions of each asset

```shell
$ apple-photos-export export <LIBRARY_PATH> <OUTPUT_DIR> -MHrfe
```

##### Only include assets from a list of specific albums

- Exports all assets that _are_ part of any of the given albums (in this case `700` and `701`)
    - Album IDs can be obtained via the `list-albums` command

```shell
$ apple-photos-export export <LIBRARY_PATH> <OUTPUT_DIR> -i 700 701
```

##### Exclude all assets being in a list of specific albums

- Exports all assets that _are not_ part of any of the given albums (in this case `700` and `701`)
    - Album IDs can be obtained via the `list-albums` command

```shell
$ apple-photos-export export <LIBRARY_PATH> <OUTPUT_DIR> -x 700 701
```

##### Export hidden files only

- Exports all _hidden_ assets

```shell
$ apple-photos-export export [library_path] [output_path] --must-be-hidden
```

</details>

## Changelog

For an overview of the changes made between each version, please have a look at the [CHANGELOG](CHANGELOG.md).

## Compatibility

The program has been tested on the following OSs with the following versions of the Photos app:

| OS                  | Photos Version                        |
|---------------------|---------------------------------------|
| macOS 14.0 (Sonoma) | 9.0 (608.2.113), <br> 9.0 (621.0.110) |

> [!NOTE]
> The above table only contains vesion that I explicitly tested.
> However, there's a high chance this tool works on other versions too.
