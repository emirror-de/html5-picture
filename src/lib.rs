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

use {
    walkdir::DirEntry,
    std::path::PathBuf,
};



/// Generates the output directory name.
pub fn get_output_dir_name(input_dir: &str) -> Result<PathBuf, String> {
    let mut path = std::path::PathBuf::from(input_dir);
    if let None = &path.file_name() {
        return Err(String::from("The last segment of the path is not valid!"));
    };
    let new_file_name = path.file_name().unwrap().to_str();
    let new_file_name = match new_file_name {
        Some(s) => String::from(s), // prevents issue #59159
        None => return Err(String::from("Failed to convert last segment of input dir to string")),
    };
    path.set_file_name(&format!("{}-html5picture", new_file_name));
    Ok(path)
}

/// Creates the output directory on the filesystem.
pub fn create_output_dir(input_dir: &str) -> Result<(), String> {
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
pub fn is_png(input: &DirEntry) -> bool {
    input.file_name()
        .to_str()
        .map(|s| s.ends_with(".png"))
        .unwrap_or(false)
}
