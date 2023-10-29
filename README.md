# Apple Photos Library Exporter

This program exports all images and videos from an Apple Photos Library to a local directory, preserving the original
album structure and making the files easily accessible without the need to use the native Photos app.
It is intended for backup purposes and does not modify the library in any way.

The project works by reverse-engineering the Apple Photos Library database and file structure. Thus, it is not
guaranteed to work with future versions of the Photos app or at all. Use it at your own risk and always keep a backup
of your library.

## Compatibility

The program has been tested on the following OSs with the following versions of the Photos app:

| OS                  | Photos Version  |
|---------------------|-----------------|
| macOS 14.0 (Sonoma) | 9.0 (608.2.113) |

## Highlights

- Exports all images and videos from the Photos Library to a local directory
- Preserves the original album structure
- Optionally restores the original file names that were used when importing the files into the library
- Dry-run mode to test the export without actually copying any files

## Usage

In order to use the program, you need to have Python 3 installed on your system. You can then install the program
by cloning the repository. All dependencies will be installed upon first execution.

You can run the program by executing the `photoslibraryexporter` script with ZSH (default shell on macOS).
Run `./photoslibraryexporter --help` to see all available options.

Here is an example command that simulates an export of all images and videos from the Photos Library to the 
`~/Pictures/Export` directory and restores the original file names:

```shell
./photoslibrary-exporter export \
    --restore-original-filenames --dry-run \
    ~/Pictures/Photos\ Library.photoslibrary ~/Desktop/Test
```