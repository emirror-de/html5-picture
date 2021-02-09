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

/// Contains functionality for scaling and saving the image files.
mod processing;

pub use processing::ImageProcessor;

use std::path::{Path, PathBuf};

/// Generates the output directory name by attaching a fixed string to it.
/// ## Example
///
/// ```
/// use {
///     html5_picture::get_output_dir_name,
///     std::path::PathBuf,
/// };
///
/// let input = PathBuf::from("../assets");
/// let input = get_output_dir_name(&input).unwrap();
/// assert_eq!(input.to_str().unwrap(), "../assets-html5picture");
/// ```
pub fn get_output_dir_name(input_dir: &PathBuf) -> Result<PathBuf, String> {
    // validity checks
    if !&input_dir.is_dir() {
        match &input_dir.to_str() {
            Some(v) => return Err(format!("{} is not a valid directory!", v)),
            None => return Err(format!("Please provide a valid directory!")),
        }
    }
    if let None = &input_dir.file_name() {
        return Err(String::from(
            "The last segment of the input path is not valid!",
        ));
    };
    // get input directory name
    let input_dir_name = input_dir.file_name().unwrap();
    // get parent directiory
    let parent = match input_dir.parent() {
        Some(p) => p,
        None => Path::new(""),
    };
    // generate output directory
    Ok(parent
        .join(format!("{}-html5picture", input_dir_name.to_str().unwrap())))
}

/// Removes the given base directory from the given input file.
/// ## Example
///
/// ```
/// use {
///     html5_picture::get_output_dir_name,
///     std::path::PathBuf,
/// };
///
/// let input = PathBuf::from("../assets");
/// let input = html5_picture::get_output_dir_name(&input).unwrap();
/// assert_eq!(input.to_str().unwrap(), "../assets-html5picture");
/// ```
pub fn remove_base_dir(
    base_dir: &PathBuf,
    input_file: &PathBuf,
) -> Result<PathBuf, String> {
    match pathdiff::diff_paths(input_file, base_dir) {
        Some(p) => Ok(p),
        None => match input_file.to_str() {
            Some(v) => {
                Err(String::from(format!("Could not remove base dir of {}", v)))
            }
            None => Err(String::from("Could not remove base dir!")),
        },
    }
}

/// Calls ```get_output_dir_name``` for name conversion and creates the output directory on the filesystem.
pub fn create_output_dir(input_dir: &PathBuf) -> Result<(), String> {
    let path = get_output_dir_name(input_dir)?;
    if path.exists() {
        return Ok(());
    }
    match std::fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(msg) => Err(msg.to_string()),
    }
}

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
