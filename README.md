# apple-photos-export

![Rust](https://img.shields.io/badge/Rust-d6a672?style=for-the-badge&logo=rust)

This program exports all images and videos from an Apple Photos Library to a local directory, making the files easily
accessible without the need to use the native Photos app.
It is intended for backup purposes and does not modify the library in any way.

> [!IMPORTANT]
> The project works by reverse-engineering the Apple Photos Library database and file structure. Thus, it is not
> guaranteed to work with future versions of the Photos app or at all. Use it at your own risk and always keep a backup
> of your library.

## Highlights

- Lists all albums of the library as an ascii tree
- Exports all images and videos from the Photos Library to a local directory
- Different export album structures are supported (overview below)
- Optionally restores the original file names that were used when importing the files into the library
- Dry-run mode to test the export without actually copying any files

### Export Album Structures

| Structure        | Description                                                                                                                                                                          |
|------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Flat             | All images and videos are exported to the same directory                                                                                                                             |
| Album            | Images and videos are exported to a directory structure that reflects the album structure in the Photos Library                                                                      |
| Year/Month       | Images and videos are exported to a directory structure that reflects the year and month of the creation date of the images and videos                                               |
| Year/Month/Album | Images and videos are exported to a directory structure that reflects the year and month of the creation date of the images and videos and the album structure in the Photos Library |

## Database / Library file structure

The Photos Library is a package that contains an SQLite database and a directory structure with the actual image and 
video files. The database contains information about the images and videos, such as the creation date, file name, and
album structure. The directory structure contains the actual image and video files, which are organized in a way that
is not directly accessible by the user.

The exact structure of the Photos Library package is not documented by Apple and may change with future versions of the
Photos app. However, this project has a wiki that documents the structure of the library package as of the time of
writing:

[📖 Wiki](https://github.com/haukesomm/apple-photos-export/wiki)

## Compatibility

The program has been tested on the following OSs with the following versions of the Photos app:

| OS                  | Photos Version                        |
|---------------------|---------------------------------------|
| macOS 14.0 (Sonoma) | 9.0 (608.2.113), <br> 9.0 (621.0.110) |

> [!NOTE]
> The above table only contains vesion that I explicitly tested.
> However, there's a high chance this tool works on other versions too.

## Usage

Example:
```shell
$ cargo build --release
$ ./target/release/apple-photos-export --help
```