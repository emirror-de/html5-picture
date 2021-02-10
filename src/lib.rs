//! # Purpose and usage
//!
//! Supports the binary within this package.
//! Contains functions to easily generate different sizes of a picture that
//! is used on webpages. Also offers the possibility to convert them into webp
//! format and is able to create a ```<picture>``` tag for the given images.
//!
//! Currently this crate is only capable of converting ```png``` files to webp using
//! ```cwebp```.
//! So make sure that webp is installed on your computer.
//!
//! ## Installation
//!
//! The binary can be installed via ```cargo install html5-picture```. As stated
//! before, make sure webp is installed before using.

/// Support for webp format. Used mainly for conversion.
pub mod webp;

/// Path processing that is required for ```html5_picture```
pub mod path;

/// Functions operating on the filesystem that is required for ```html5_picture```
pub mod fs;

use std::path::PathBuf;

/// Determines if the given input filename contains a .png extension.
pub fn is_png(input: &PathBuf) -> bool {
    match input.extension() {
        Some(s) => match s.to_str() {
            None => false,
            Some(v) => v == "png",
        },
        None => false,
    }
}
