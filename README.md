# html5-picture

Batch optimizes your images to webp format and generates HTML5 picture tags for responsive web design.

## Purpose

This tool supports easy generation of different sizes of images that can be used on webpages. It converts images to webp format and creates `<picture>` tags for the given images with multiple responsive breakpoints.

Currently this crate is only capable of converting `png` files to webp format using the `webp` library. Make sure that webp is installed on your computer.

## Installation

The binary can be installed via:

```bash
cargo install html5-picture
```

**Prerequisites:** Make sure webp is installed on your system before using this tool.

## Usage

Use `html5-picture --help` for an overview of all parameters.

### Basic Syntax

```bash
html5-picture <INPUT_DIR> <SCALED_IMAGES_COUNT> [OPTIONS]
```

### Arguments

- `INPUT_DIR` - The directory containing all images that should be processed
- `SCALED_IMAGES_COUNT` - The source image width is divided by this value + 1. The source image is then scaled (keeping aspect ratio) to these widths before conversion. Useful for creating multiple sizes for different responsive breakpoints.

### Options

- `-i <folder>` - Installs the converted and sized pictures into the given folder
- `-p <folder>` - The destination folder of HTML5 picture tag files  
- `-m <path>` - Sets the mountpoint for links in the HTML tags (use with `-p`)
- `-q <quality>` - Defines the quality of webp conversion (0-100)
- `-f, --force-overwrite` - Overwrites existing files if they exist
- `-s` - Process images single-threaded instead of multi-threaded

## Examples

### Basic conversion with three scales and 70% quality

Convert images in `./assets` to create three different sizes with 70% quality:

```bash
html5-picture ./assets 3 -q 70
```

This will convert your images and save them to `./assets-html5picture`. This folder is the working directory - make sure not to modify it while the application is running.

**Example output for a 6000x962 input image:**
- `original_filename.webp` (6000x962)
- `original_filename-w4500.webp` (4500x751) 
- `original_filename-w3000.webp` (3000x501)
- `original_filename-w1500.webp` (1500x250)

### Conversion with custom installation folder

Move resulting files to a specific directory after conversion:

```bash
html5-picture ./assets 3 -q 100 -i ./assets-build
```

The converted images will be installed to `./assets-build`.

### Force overwriting existing files

Overwrite existing webp or HTML5 picture tag files:

```bash
html5-picture ./assets 3 -q 100 -i ./dist -f
```

### Generate HTML5 picture tags

Save `<picture>` tags to disk with web server mountpoint:

```bash
html5-picture ./assets 4 -i ./dist -p ./html5-tags -m /some/web-server/mountpoint
```

This creates HTML5 `<picture>` tags that reference the converted images with the specified mountpoint path.

## Output

The tool generates:

1. **Responsive webp images** - Multiple sizes of each input image optimized for web delivery
2. **HTML5 picture tags** (optional) - Ready-to-use `<picture>` elements for responsive images in web pages
3. **Organized directory structure** - Clean separation of original and converted files

## Documentation

For detailed API documentation, visit [docs.rs/html5-picture](https://docs.rs/html5-picture).

## Current Limitations

- Only supports PNG input files
- Requires webp to be installed on the system
- Single format output (webp only)

## License

MIT