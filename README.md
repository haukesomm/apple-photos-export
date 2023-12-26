# Apple Photos Library Exporter

This program exports all images and videos from an Apple Photos Library to a local directory, making the files easily 
accessible without the need to use the native Photos app.
It is intended for backup purposes and does not modify the library in any way.

The project works by reverse-engineering the Apple Photos Library database and file structure. Thus, it is not
guaranteed to work with future versions of the Photos app or at all. Use it at your own risk and always keep a backup
of your library.

## Highlights

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

## Compatibility

The program has been tested on the following OSs with the following versions of the Photos app:

| OS                  | Photos Version  |
|---------------------|-----------------|
| macOS 14.0 (Sonoma) | 9.0 (608.2.113) |

## Usage

In order to use the program, you need to have Python 3 installed on your system. You can then install the program
by cloning the repository. All dependencies will be installed upon first execution.

You can run the program by executing the `photoslibrary-exporter` script with ZSH (default shell on macOS).
Run `./photoslibrary-exporter --help` to see all available options.

Here is an example command that simulates an export of all images and videos from the Photos Library to the 
`~/Pictures/Export` directory and restores the original file names:

```shell
./apple-photos-export export \
    --restore-original-filenames --dry-run \
    ~/Pictures/Photos\ Library.photoslibrary ~/Desktop/Test
```