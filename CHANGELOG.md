# Changelog

## 🎉 v0.2.3

### 📦 New features

### 📈 Changes

### 🐛 Bugfixes

* The copying process of images is no longer aborted. The images already existing are skipped if `-f` flag is not specified

### 🔨 Breaking changes

## 🎉 v0.2.2

Bugfix release.  The following adjustments have been made:

* `clap` dependency has been upgraded to `3.0.5` that introduced a breaking change.

* All dependencies are now fixed instead of having the prefix `^` to make sure the program works as expected.

* During processing, the prefix of the progressbar now shows the file name instead of the full path, enabling to have a smaller terminal width without getting visual artifacts.

## 🎉 v0.2.0

Single threaded processing option has been added. Please note, that if using it with pictures having unsupported color profiles, an error is shown in the console, as well as a `panic` note of the main thread. This *does not* abort the processing! This behavior is due to [`webp`](https://github.com/jaredforth/webp) currently runs in an `unreachable!` statement if a color profile is not supported.

### Added features

* The following flags have been implemented
  * `-s`, If set, the processing is done single threaded

## 🎉 v0.1.0

### Usage of the command line utility

`html5-picture [FLAGS] [OPTIONS] <INPUT_DIR> <SCALED_IMAGES_COUNT> `

Invoke `html5-picture --help`for more information.

### Added features

* Added basic functionality, the following arguments are required:
  * `INPUT_DIR`, The directory containing all images that should be processed
  * `SCALED_IMAGES_COUNT`, The source image width is divided by this option (value + 1). Afterwards the source image is scaled (keeping the aspect ratio) to these widths before convertion. Useful if you want to have multiple sizes of the image on the webpage for different breakpoints
* The following flags have been implemented
  * `-f`, `--force-overwrite`,  If given, existing files are overwritten (if option `install-images-into` is set)
* The following options have been implemented
  * `-i`,  Installs the converted and sized pictures into the given folder
  * `-m`, Can be used in combination with `-p`, sets the mountpoint for links in the HTML tags
  * `-p`, The destination folder of HTML5 picture tag files
  * `-q`, Defines the quality of cwebp conversion